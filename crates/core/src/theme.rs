//! The theme interface and the writer for a complete Xcursor theme:
//! ordinary cursors, gestures, reversible transitions and aliases.
//!
//! Every image is exactly its nominal size. One scale per theme is fitted so
//! the largest frame, shadow and animations included, fills the image, and
//! each cursor's hotspot is placed to center its frames.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use tiny_skia::{Pixmap, Transform};

use crate::paint::argb;
use crate::palette::Palette;
use crate::shape::{LEGACY_ALIASES, PAIRS, Shape};
use crate::xcursor::{Image, crop, encode};

/// Nominal sizes built by default.
pub const SIZES: [u32; 5] = [32, 48, 64, 72, 96];
pub const TRANSITION_FRAMES: usize = 24;
pub const TRANSITION_DELAY: u32 = 5;
/// Pixels kept clear around the largest frame.
const MARGIN: f32 = 2.0;
/// Pixels per design unit of the render that measures frames, and how far it
/// reaches around the hotspot in design units.
const PROBE: f32 = 4.0;
const REACH: f32 = 32.0;
/// Alpha above which a pixel counts as painted; fainter shadow tails may clip.
pub const PAINTED: u8 = 16;

pub struct Frame<S> {
    pub scene: S,
    /// Milliseconds; 0 for a static cursor.
    pub delay: u32,
}

/// What a cursor file shows: an ordinary shape, or one of a theme's own
/// gestures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Look {
    Shape(Shape),
    Gesture(&'static str),
}

impl Look {
    /// File name of the cursor.
    pub fn name(self) -> &'static str {
        match self {
            Look::Shape(shape) => shape.name(),
            Look::Gesture(name) => name,
        }
    }

    /// Busy cursors, which loop while an application works.
    pub fn is_busy(self) -> bool {
        matches!(self, Look::Shape(shape) if shape.is_busy())
    }
}

/// A shape overridden by another cursor: it installs as a link to `target`,
/// and its transitions start and end on `target`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alias {
    pub shape: Shape,
    pub target: Look,
}

impl Alias {
    /// Parses `shape=cursor,...`, each cursor a shape or one of `T::GESTURES`.
    /// Each shape is aliased at most once and never to an aliased shape.
    pub fn parse<T: Theme>(list: &str) -> Result<Vec<Alias>, String> {
        let mut aliases: Vec<Alias> = Vec::new();
        for entry in list.split(',').map(str::trim).filter(|entry| !entry.is_empty()) {
            let (shape, target) =
                entry.split_once('=').ok_or_else(|| format!("Alias {entry:?} is not shape=cursor"))?;
            let shape = Shape::from_name(shape.trim()).ok_or_else(|| format!("Unknown cursor {shape:?}"))?;
            let target = match T::GESTURES.iter().find(|&&name| name == target.trim()) {
                Some(gesture) => Look::Gesture(gesture),
                None => Look::Shape(Shape::from_name(target.trim()).ok_or_else(|| match T::GESTURES {
                    [] => format!("Unknown cursor {target:?}"),
                    gestures => format!("Unknown cursor {target:?}; {} also offers [{}]", T::ID, gestures.join(", ")),
                })?),
            };
            if target == Look::Shape(shape) || aliases.iter().any(|alias| alias.shape == shape) {
                return Err(format!("{} is aliased twice or to itself", shape.name()));
            }
            aliases.push(Alias { shape, target });
        }
        match aliases.iter().find(|alias| aliases.iter().any(|other| alias.target == Look::Shape(other.shape))) {
            Some(alias) => {
                Err(format!("{} is aliased to {}, which is aliased itself", alias.shape.name(), alias.target.name()))
            }
            None => Ok(aliases),
        }
    }
}

/// What `shape` shows under `aliases`.
pub fn look(shape: Shape, aliases: &[Alias]) -> Look {
    aliases.iter().find(|alias| alias.shape == shape).map_or(Look::Shape(shape), |alias| alias.target)
}

/// Every cursor drawn into its own file: the theme's gestures and the shapes
/// not aliased away.
pub fn looks<T: Theme>(aliases: &[Alias]) -> Vec<Look> {
    let shapes = Shape::ALL.into_iter().filter(|&shape| look(shape, aliases) == Look::Shape(shape)).map(Look::Shape);
    shapes.chain(T::GESTURES.iter().map(|&name| Look::Gesture(name))).collect()
}

/// Every name a shape's transitions are installed under, canonical first:
/// its own, its aliases' and the gesture it is aliased to.
fn names(shape: Shape, aliases: &[Alias]) -> impl Iterator<Item = &'static str> + '_ {
    let gesture = match look(shape, aliases) {
        Look::Gesture(name) => Some(name),
        Look::Shape(_) => None,
    };
    std::iter::once(shape.name()).chain(shape.aliases().iter().copied()).chain(gesture)
}

pub trait Theme: Sync {
    /// Whatever the theme needs to draw one frame.
    type Scene: Send + Sync;

    /// Installed directory name.
    const ID: &'static str;
    const TITLE: &'static str;

    /// Color roles every palette defines.
    const ROLES: &'static [&'static str];
    /// Bundled palettes as (name, `role = #rrggbb` text); the first is the
    /// default.
    const PALETTES: &'static [(&'static str, &'static str)];

    /// Gestures of the theme's own, each installed as a cursor of its name.
    const GESTURES: &'static [&'static str] = &[];

    /// The theme drawn in `palette`.
    fn new(palette: Palette) -> Self;

    fn palette(&self) -> &Palette;

    /// Frames of an ordinary cursor; more than one makes it loop.
    fn cursor(&self, shape: Shape) -> Vec<Frame<Self::Scene>>;

    /// Play order of an ordinary cursor's frames, as indices into `cursor`.
    /// Frames may repeat, so a long loop stores each distinct frame once.
    fn order(&self, shape: Shape) -> Vec<usize> {
        (0..self.cursor(shape).len()).collect()
    }

    /// Frames of a gesture, one of `GESTURES`, as `cursor`.
    fn gesture(&self, name: &'static str) -> Vec<Frame<Self::Scene>> {
        unreachable!("{} has no gesture {name}", Self::ID)
    }

    /// Play order of a gesture's frames, as `order`.
    fn gesture_order(&self, name: &'static str) -> Vec<usize> {
        (0..self.gesture(name).len()).collect()
    }

    /// The scene at progress `t` in (0, 1) of the transition from `from` to
    /// `to`. Both endpoints are the cursors' first frames. Playing it
    /// backwards should match the transition from `to` to `from`.
    fn transition(&self, from: Shape, to: Shape, t: f32) -> Self::Scene;

    /// As `transition`, between any two looks; themes with gestures override it.
    fn morph(&self, from: Look, to: Look, t: f32) -> Self::Scene {
        match (from, to) {
            (Look::Shape(from), Look::Shape(to)) => self.transition(from, to, t),
            _ => unreachable!("{} has no gestures", Self::ID),
        }
    }

    /// Draws a scene in design units with the hotspot at the origin.
    fn draw(&self, scene: &Self::Scene, pixmap: &mut Pixmap, transform: Transform);

    /// As `draw`, where only the coverage counts, so themes may skip shading.
    fn silhouette(&self, scene: &Self::Scene, pixmap: &mut Pixmap, transform: Transform) {
        self.draw(scene, pixmap, transform);
    }
}

/// Frames of a look, as `Theme::cursor`.
pub fn frames<T: Theme>(theme: &T, look: Look) -> Vec<Frame<T::Scene>> {
    match look {
        Look::Shape(shape) => theme.cursor(shape),
        Look::Gesture(name) => theme.gesture(name),
    }
}

/// Play order of a look's frames, as `Theme::order`.
pub fn order<T: Theme>(theme: &T, look: Look) -> Vec<usize> {
    match look {
        Look::Shape(shape) => theme.order(shape),
        Look::Gesture(name) => theme.gesture_order(name),
    }
}

/// Box in design units around the hotspot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Bounds {
    const EMPTY: Bounds = Bounds { x0: f32::INFINITY, y0: f32::INFINITY, x1: f32::NEG_INFINITY, y1: f32::NEG_INFINITY };

    fn union(self, other: Bounds) -> Bounds {
        Bounds {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    /// Longer side, or 0 when empty.
    fn extent(&self) -> f32 {
        (self.x1 - self.x0).max(self.y1 - self.y0).max(0.0)
    }

    fn center(&self) -> (f32, f32) {
        if self.x0 > self.x1 { (0.0, 0.0) } else { ((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0) }
    }
}

/// Bounds of everything a scene paints, shadow included.
pub fn measure<T: Theme>(theme: &T, scene: &T::Scene) -> Bounds {
    let side = (2.0 * REACH * PROBE) as usize;
    let center = side as f32 / 2.0;
    let mut pixmap = Pixmap::new(side as u32, side as u32).expect("nonzero size");
    theme.silhouette(scene, &mut pixmap, Transform::from_row(PROBE, 0.0, 0.0, PROBE, center, center));
    let unit = |pixel: usize| (pixel as f32 - center) / PROBE;
    pixmap.pixels().iter().enumerate().filter(|(_, pixel)| pixel.alpha() > PAINTED).fold(
        Bounds::EMPTY,
        |bounds, (index, _)| {
            let (x, y) = (index % side, index / side);
            bounds.union(Bounds { x0: unit(x), y0: unit(y), x1: unit(x + 1), y1: unit(y + 1) })
        },
    )
}

/// Frames with the bounds each frame's hotspot is placed to center, and
/// the order they play in.
pub struct Sequence<S> {
    pub frames: Vec<Frame<S>>,
    pub bounds: Vec<Bounds>,
    pub order: Vec<usize>,
}

/// One scale for a whole theme, fitted to its largest frame.
#[derive(Clone, Copy, Debug)]
pub struct Fit {
    /// Longest side of the largest frame in design units.
    pub extent: f32,
}

impl Fit {
    /// Pixels per design unit at nominal `size`.
    pub fn scale(&self, size: u32) -> f32 {
        (size as f32 - 2.0 * MARGIN) / self.extent
    }

    /// Hotspot pixel that centers `bounds` in a `size` square.
    pub fn hotspot(&self, bounds: &Bounds, size: u32) -> (u32, u32) {
        let (x, y) = bounds.center();
        let pixel = |c: f32| (size as f32 / 2.0 - 0.5 - c * self.scale(size)).round().clamp(0.0, size as f32 - 1.0);
        (pixel(x) as u32, pixel(y) as u32)
    }
}

/// Each cursor's sequence, drawn by its theme: every frame of a cursor shares
/// one hotspot, centered on all of them. Each frame is measured as its own
/// job, so long loops spread across threads.
pub fn cursor_sequences<T: Theme>(jobs: &[(&T, Look)]) -> Vec<Sequence<T::Scene>> {
    let mut sequences = parallel(jobs, |&(theme, look)| frames(theme, look));
    let frames: Vec<(usize, usize)> =
        sequences.iter().enumerate().flat_map(|(i, frames)| (0..frames.len()).map(move |j| (i, j))).collect();
    let mut bounds = parallel(&frames, |&(i, j)| measure(jobs[i].0, &sequences[i][j].scene)).into_iter();
    jobs.iter()
        .zip(sequences.iter_mut())
        .map(|(&(theme, look), frames)| {
            let union = bounds.by_ref().take(frames.len()).fold(Bounds::EMPTY, Bounds::union);
            let frames = std::mem::take(frames);
            Sequence { bounds: vec![union; frames.len()], frames, order: order(theme, look) }
        })
        .collect()
}

/// The transition between each pair. Endpoints are the cursors' first frames
/// with their hotspots, so they match the ordinary cursors exactly; frames
/// between center themselves. Each frame is its own job.
pub fn transition_sequences<T: Theme>(
    theme: &T,
    pairs: &[(Look, Look)],
    cursors: &Cursors<T::Scene>,
) -> Vec<Sequence<T::Scene>> {
    let jobs: Vec<(usize, usize)> =
        (0..pairs.len()).flat_map(|i| (0..TRANSITION_FRAMES).map(move |j| (i, j))).collect();
    let mut steps = parallel(&jobs, |&(i, j)| transition_frame(theme, pairs[i], cursors, j)).into_iter();
    (0..pairs.len())
        .map(|_| {
            let (frames, bounds) = steps.by_ref().take(TRANSITION_FRAMES).unzip();
            Sequence { frames, bounds, order: (0..TRANSITION_FRAMES).collect() }
        })
        .collect()
}

/// Frame `index` of a transition with the bounds its hotspot centers.
fn transition_frame<T: Theme>(
    theme: &T,
    (from, to): (Look, Look),
    cursors: &Cursors<T::Scene>,
    index: usize,
) -> (Frame<T::Scene>, Bounds) {
    let last = TRANSITION_FRAMES - 1;
    let first = |look: Look| frames(theme, look).into_iter().next().expect("cursor has frames").scene;
    let (scene, bounds) = match index {
        0 => (first(from), cursors[&from].bounds[0]),
        _ if index == last => (first(to), cursors[&to].bounds[0]),
        _ => {
            let scene = theme.morph(from, to, index as f32 / last as f32);
            let bounds = measure(theme, &scene);
            (scene, bounds)
        }
    };
    (Frame { scene, delay: TRANSITION_DELAY }, bounds)
}

/// Renders a scene on a `size` square at `scale` pixels per design unit, with
/// the hotspot at the center of pixel `hotspot`.
pub fn render<T: Theme>(theme: &T, scene: &T::Scene, scale: f32, size: u32, hotspot: (u32, u32)) -> Pixmap {
    let (x, y) = (hotspot.0 as f32 + 0.5, hotspot.1 as f32 + 0.5);
    let mut pixmap = Pixmap::new(size, size).expect("nonzero size");
    theme.draw(scene, &mut pixmap, Transform::from_row(scale, 0.0, 0.0, scale, x, y));
    pixmap
}

/// Frame `index` of a sequence at nominal `size`, with its hotspot.
fn pixmap<T: Theme>(
    theme: &T,
    sequence: &Sequence<T::Scene>,
    fit: Fit,
    size: u32,
    index: usize,
) -> (Pixmap, (u32, u32)) {
    let hotspot = fit.hotspot(&sequence.bounds[index], size);
    (render(theme, &sequence.frames[index].scene, fit.scale(size), size, hotspot), hotspot)
}

/// Every frame of each sequence, drawn by its theme at nominal `size` with
/// its hotspot. Each frame is its own job.
pub(crate) fn pixmaps<T: Theme>(
    jobs: &[(&T, &Sequence<T::Scene>)],
    fit: Fit,
    size: u32,
) -> Vec<Vec<(Pixmap, (u32, u32))>> {
    let frames: Vec<(usize, usize)> = jobs
        .iter()
        .enumerate()
        .flat_map(|(i, (_, sequence))| (0..sequence.frames.len()).map(move |j| (i, j)))
        .collect();
    let mut drawn = parallel(&frames, |&(i, j)| pixmap(jobs[i].0, jobs[i].1, fit, size, j)).into_iter();
    jobs.iter().map(|(_, sequence)| drawn.by_ref().take(sequence.frames.len()).collect()).collect()
}

/// Frame `index` of a sequence as an Xcursor image at nominal `size`.
pub fn image<T: Theme>(theme: &T, sequence: &Sequence<T::Scene>, fit: Fit, size: u32, index: usize) -> Image {
    let (pixmap, (xhot, yhot)) = pixmap(theme, sequence, fit, size, index);
    Image { size, width: size, height: size, xhot, yhot, delay: sequence.frames[index].delay, pixels: argb(&pixmap) }
}

pub fn images<T: Theme>(theme: &T, sequence: &Sequence<T::Scene>, fit: Fit, size: u32) -> Vec<Image> {
    (0..sequence.frames.len()).map(|index| image(theme, sequence, fit, size, index)).collect()
}

/// Every name for a transition, canonical first.
pub fn transition_names(from: Shape, to: Shape, aliases: &[Alias]) -> Vec<String> {
    names(from, aliases).flat_map(|a| names(to, aliases).map(move |b| format!("{a}-to-{b}"))).collect()
}

/// Links as (name, target): each transition under its other names, the first
/// pair in `PAIRS` taking a name two would share.
pub fn transition_links(aliases: &[Alias]) -> Vec<(String, String)> {
    let mut taken: HashSet<String> = PAIRS.iter().map(|&(a, b)| format!("{}-to-{}", a.name(), b.name())).collect();
    PAIRS
        .iter()
        .flat_map(|&(a, b)| {
            let mut names = transition_names(a, b, aliases).into_iter();
            let canonical = names.next().expect("canonical name");
            names.map(move |name| (name, canonical.clone()))
        })
        .filter(|(name, _)| taken.insert(name.clone()))
        .collect()
}

/// Links as (name, target): aliased shapes to their targets, and the other
/// names of every shape to it.
pub fn cursor_links(aliases: &[Alias]) -> Vec<(&'static str, &'static str)> {
    let overridden = aliases.iter().map(|alias| (alias.shape.name(), alias.target.name()));
    let named = Shape::ALL.into_iter().flat_map(|shape| shape.aliases().iter().map(move |name| (*name, shape.name())));
    let legacy =
        LEGACY_ALIASES.into_iter().flat_map(|(shape, names)| names.iter().map(move |name| (*name, shape.name())));
    overridden.chain(named).chain(legacy).collect()
}

/// Maps jobs across threads, keeping their order.
pub(crate) fn parallel<J: Sync, R: Send>(jobs: &[J], work: impl Fn(&J) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let workers = thread::available_parallelism().map_or(4, |n| n.get());
    let mut results: Vec<(usize, R)> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(job) = jobs.get(index) else { return done };
                        done.push((index, work(job)));
                    }
                })
            })
            .collect();
        handles.into_iter().flat_map(|handle| handle.join().expect("worker panicked")).collect()
    });
    results.sort_unstable_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

pub type Cursors<S> = HashMap<Look, Sequence<S>>;

/// Every cursor drawn into its own file and every transition in `PAIRS`
/// order, between the looks in `pairs`, with the one scale that fits them all.
pub struct Fitted<S> {
    pub cursors: Cursors<S>,
    pub pairs: Vec<(Look, Look)>,
    pub transitions: Vec<Sequence<S>>,
    pub fit: Fit,
}

/// Every cursor and transition, with the one scale that fits them all.
pub fn fitted<T: Theme>(theme: &T, aliases: &[Alias]) -> Fitted<T::Scene> {
    let looks = looks::<T>(aliases);
    let jobs: Vec<(&T, Look)> = looks.iter().map(|&look| (theme, look)).collect();
    let cursors: Cursors<T::Scene> = looks.into_iter().zip(cursor_sequences(&jobs)).collect();
    let pairs: Vec<(Look, Look)> = PAIRS.iter().map(|&(a, b)| (look(a, aliases), look(b, aliases))).collect();
    let transitions = transition_sequences(theme, &pairs, &cursors);
    let extent = cursors.values().chain(&transitions).flat_map(|s| &s.bounds).map(Bounds::extent).fold(0.0, f32::max);
    Fitted { cursors, pairs, transitions, fit: Fit { extent } }
}

pub fn generate<T: Theme>(theme: &T, output: &Path, sizes: &[u32], aliases: &[Alias]) -> io::Result<()> {
    let directory = output.join("cursors");
    fs::create_dir_all(&directory)?;
    fs::write(output.join("index.theme"), format!("[Icon Theme]\nName={} ({})\n", T::TITLE, theme.palette().name))?;

    let Fitted { cursors, transitions, fit, .. } = fitted(theme, aliases);
    let jobs: Vec<(String, &Sequence<T::Scene>)> = cursors
        .iter()
        .map(|(look, sequence)| (look.name().to_owned(), sequence))
        .chain(PAIRS.into_iter().zip(&transitions).map(|((a, b), s)| (format!("{}-to-{}", a.name(), b.name()), s)))
        .collect();
    // Every image is its own job, so long loops spread across threads.
    let renders: Vec<(usize, u32, usize)> = jobs
        .iter()
        .enumerate()
        .flat_map(|(i, (_, sequence))| {
            sizes.iter().flat_map(move |&size| (0..sequence.frames.len()).map(move |j| (i, size, j)))
        })
        .collect();
    let mut rendered = parallel(&renders, |&(i, size, j)| image(theme, jobs[i].1, fit, size, j)).into_iter();
    let files: Vec<(&String, _, Vec<Image>)> = jobs
        .iter()
        .map(|(name, sequence)| {
            let mut images: Vec<Image> = rendered.by_ref().take(sizes.len() * sequence.frames.len()).collect();
            images.chunks_mut(sequence.frames.len()).for_each(crop);
            (name, *sequence, images)
        })
        .collect();
    parallel(&files, |(name, sequence, images)| {
        // Each size plays the frames in order, repeats pointing at one image.
        let count = sequence.frames.len();
        let entries: Vec<usize> =
            (0..sizes.len()).flat_map(|block| sequence.order.iter().map(move |&i| block * count + i)).collect();
        fs::write(directory.join(name), encode(images, &entries))
    })
    .into_iter()
    .collect::<io::Result<()>>()?;

    for (name, target) in transition_links(aliases) {
        symlink(target, directory.join(name))?;
    }
    for (name, target) in cursor_links(aliases) {
        symlink(target, directory.join(name))?;
    }
    Ok(())
}
