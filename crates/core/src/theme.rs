//! The theme interface and the writer for a complete Xcursor theme:
//! ordinary cursors, reversible transitions, aliases and preview GIFs.
//!
//! Every image is exactly its nominal size. One scale per theme is fitted so
//! the largest frame, shadow and animations included, fills the image, and
//! each cursor's hotspot is placed to center its frames.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use tiny_skia::{Pixmap, PixmapPaint, Transform};

use crate::paint::argb;
use crate::shape::{LEGACY_ALIASES, PAIRS, Shape};
use crate::xcursor::{Image, encode};

/// Nominal sizes built by default.
pub const SIZES: [u32; 2] = [48, 72];
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

pub trait Theme: Sync {
    /// Whatever the theme needs to draw one frame.
    type Scene: Send + Sync;

    /// Installed directory name.
    const ID: &'static str;
    const TITLE: &'static str;

    /// Frames of an ordinary cursor; more than one makes it loop.
    fn cursor(&self, shape: Shape) -> Vec<Frame<Self::Scene>>;

    /// The scene at progress `t` in (0, 1) of the transition from `from` to
    /// `to`. Both endpoints are the cursors' first frames. Playing it
    /// backwards should match the transition from `to` to `from`.
    fn transition(&self, from: Shape, to: Shape, t: f32) -> Self::Scene;

    /// Draws a scene in design units with the hotspot at the origin.
    fn draw(&self, scene: &Self::Scene, pixmap: &mut Pixmap, transform: Transform);
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
    theme.draw(scene, &mut pixmap, Transform::from_row(PROBE, 0.0, 0.0, PROBE, center, center));
    let unit = |pixel: usize| (pixel as f32 - center) / PROBE;
    pixmap.pixels().iter().enumerate().filter(|(_, pixel)| pixel.alpha() > PAINTED).fold(
        Bounds::EMPTY,
        |bounds, (index, _)| {
            let (x, y) = (index % side, index / side);
            bounds.union(Bounds { x0: unit(x), y0: unit(y), x1: unit(x + 1), y1: unit(y + 1) })
        },
    )
}

/// Frames with the bounds each frame's hotspot is placed to center.
pub struct Sequence<S> {
    pub frames: Vec<Frame<S>>,
    pub bounds: Vec<Bounds>,
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

/// Every frame of a cursor shares one hotspot, centered on all of them.
pub fn cursor_sequence<T: Theme>(theme: &T, shape: Shape) -> Sequence<T::Scene> {
    let frames = theme.cursor(shape);
    let union = frames.iter().fold(Bounds::EMPTY, |bounds, frame| bounds.union(measure(theme, &frame.scene)));
    Sequence { bounds: vec![union; frames.len()], frames }
}

/// Endpoints are the cursors' first frames with their hotspots, so they match
/// the ordinary cursors exactly; frames between center themselves.
pub fn transition_sequence<T: Theme>(
    theme: &T,
    (from, to): (Shape, Shape),
    cursors: &HashMap<Shape, Sequence<T::Scene>>,
) -> Sequence<T::Scene> {
    let last = TRANSITION_FRAMES - 1;
    let first = |shape: Shape| theme.cursor(shape).into_iter().next().expect("cursor has frames").scene;
    let (frames, bounds) = (0..TRANSITION_FRAMES)
        .map(|index| {
            let (scene, bounds) = match index {
                0 => (first(from), cursors[&from].bounds[0]),
                _ if index == last => (first(to), cursors[&to].bounds[0]),
                _ => {
                    let scene = theme.transition(from, to, index as f32 / last as f32);
                    let bounds = measure(theme, &scene);
                    (scene, bounds)
                }
            };
            (Frame { scene, delay: TRANSITION_DELAY }, bounds)
        })
        .unzip();
    Sequence { frames, bounds }
}

/// Renders a scene on a `size` square at `scale` pixels per design unit, with
/// the hotspot at the center of pixel `hotspot`.
pub fn render<T: Theme>(theme: &T, scene: &T::Scene, scale: f32, size: u32, hotspot: (u32, u32)) -> Pixmap {
    let (x, y) = (hotspot.0 as f32 + 0.5, hotspot.1 as f32 + 0.5);
    let mut pixmap = Pixmap::new(size, size).expect("nonzero size");
    theme.draw(scene, &mut pixmap, Transform::from_row(scale, 0.0, 0.0, scale, x, y));
    pixmap
}

fn pixmaps<T: Theme>(theme: &T, sequence: &Sequence<T::Scene>, fit: Fit, size: u32) -> Vec<(Pixmap, (u32, u32))> {
    let scale = fit.scale(size);
    sequence
        .frames
        .iter()
        .zip(&sequence.bounds)
        .map(|(frame, bounds)| {
            let hotspot = fit.hotspot(bounds, size);
            (render(theme, &frame.scene, scale, size, hotspot), hotspot)
        })
        .collect()
}

pub fn images<T: Theme>(theme: &T, sequence: &Sequence<T::Scene>, fit: Fit, size: u32) -> Vec<Image> {
    pixmaps(theme, sequence, fit, size)
        .into_iter()
        .zip(&sequence.frames)
        .map(|((pixmap, (xhot, yhot)), frame)| Image {
            size,
            width: size,
            height: size,
            xhot,
            yhot,
            delay: frame.delay,
            pixels: argb(&pixmap),
        })
        .collect()
}

/// Every name for a transition, canonical first.
pub fn transition_names(from: Shape, to: Shape) -> Vec<String> {
    let names = |shape: Shape| std::iter::once(shape.name()).chain(shape.aliases().iter().copied());
    names(from).flat_map(|a| names(to).map(move |b| format!("{a}-to-{b}"))).collect()
}

/// Frames at 64 px placed by their hotspots on a canvas twice as wide.
/// Loops play in real time; transitions play slowed down, both ways, with
/// pauses at the endpoints.
fn preview<T: Theme>(
    theme: &T,
    sequence: &Sequence<T::Scene>,
    fit: Fit,
    transition: bool,
    path: &Path,
) -> io::Result<()> {
    const SIZE: u32 = 64;
    let pixmaps: Vec<Pixmap> = pixmaps(theme, sequence, fit, SIZE)
        .into_iter()
        .map(|(image, (xhot, yhot))| {
            let mut canvas = Pixmap::new(SIZE * 2, SIZE * 2).expect("nonzero size");
            let (x, y) = ((SIZE - xhot) as i32, (SIZE - yhot) as i32);
            canvas.draw_pixmap(x, y, image.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            canvas
        })
        .collect();
    let last = pixmaps.len() - 1;
    let sequence: Vec<(usize, u16)> = if transition {
        let hold = |index: usize, end: usize| if index == end { 40 } else { 2 };
        (0..=last).map(|i| (i, hold(i, last))).chain((0..=last).rev().map(|i| (i, hold(i, 0)))).collect()
    } else {
        sequence.frames.iter().enumerate().map(|(i, frame)| (i, (frame.delay / 10).max(2) as u16)).collect()
    };
    let (frames, delays): (Vec<&Pixmap>, Vec<u16>) =
        sequence.into_iter().map(|(i, delay)| (&pixmaps[i], delay)).unzip();
    gif(path, &frames, &delays, 1)
}

/// Writes pixmaps as a looping GIF over a neutral grey, scaled up by `scale`
/// with nearest-neighbour sampling; `delays` are in hundredths of a second.
pub fn gif(path: &Path, pixmaps: &[&Pixmap], delays: &[u16], scale: usize) -> io::Result<()> {
    const BACKGROUND: [u8; 3] = [0x55, 0x55, 0x55];
    let (width, height) = (pixmaps[0].width() as usize * scale, pixmaps[0].height() as usize * scale);
    let mut encoder =
        gif::Encoder::new(File::create(path)?, width as u16, height as u16, &[]).map_err(io::Error::other)?;
    encoder.set_repeat(gif::Repeat::Infinite).map_err(io::Error::other)?;
    for (pixmap, delay) in pixmaps.iter().zip(delays) {
        let rgb: Vec<u8> = (0..width * height)
            .flat_map(|i| {
                let pixel = pixmap.pixel((i % width / scale) as u32, (i / width / scale) as u32).unwrap();
                let alpha = 255 - pixel.alpha() as u16;
                let channels = [pixel.red(), pixel.green(), pixel.blue()];
                (0..3).map(move |c| (channels[c] as u16 + BACKGROUND[c] as u16 * alpha / 255) as u8)
            })
            .collect();
        let mut frame = gif::Frame::from_rgb_speed(width as u16, height as u16, &rgb, 10);
        frame.delay = *delay;
        encoder.write_frame(&frame).map_err(io::Error::other)?;
    }
    Ok(())
}

/// Maps jobs across threads, keeping their order.
fn parallel<J: Sync, R: Send>(jobs: &[J], work: impl Fn(&J) -> R + Sync) -> Vec<R> {
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

pub fn generate<T: Theme>(theme: &T, output: &Path, previews: Option<&Path>, sizes: &[u32]) -> io::Result<()> {
    let cursors = output.join("cursors");
    fs::create_dir_all(&cursors)?;
    if let Some(directory) = previews {
        fs::create_dir_all(directory)?;
    }
    fs::write(output.join("index.theme"), format!("[Icon Theme]\nName={}\n", T::TITLE))?;

    let shapes: HashMap<Shape, Sequence<T::Scene>> =
        Shape::ALL.into_iter().zip(parallel(&Shape::ALL, |shape| cursor_sequence(theme, *shape))).collect();
    let transitions = parallel(&PAIRS, |pair| transition_sequence(theme, *pair, &shapes));
    let extent = shapes.values().chain(&transitions).flat_map(|s| &s.bounds).map(Bounds::extent).fold(0.0, f32::max);
    let fit = Fit { extent };

    let jobs: Vec<(String, &Sequence<T::Scene>, bool)> = Shape::ALL
        .into_iter()
        .map(|shape| (shape.name().to_owned(), &shapes[&shape], false))
        .chain(
            PAIRS.into_iter().zip(&transitions).map(|((a, b), s)| (format!("{}-to-{}", a.name(), b.name()), s, true)),
        )
        .collect();
    parallel(&jobs, |(name, sequence, transition)| {
        let images: Vec<Image> = sizes.iter().flat_map(|&size| images(theme, sequence, fit, size)).collect();
        fs::write(cursors.join(name), encode(&images))?;
        match previews {
            Some(directory) => preview(theme, sequence, fit, *transition, &directory.join(format!("{name}.gif"))),
            _ => Ok(()),
        }
    })
    .into_iter()
    .collect::<io::Result<()>>()?;

    for (a, b) in PAIRS {
        let names = transition_names(a, b);
        for link in &names[1..] {
            symlink(&names[0], cursors.join(link))?;
        }
    }
    let aliases = Shape::ALL.into_iter().flat_map(|shape| shape.aliases().iter().map(move |name| (shape, *name)));
    let legacy = LEGACY_ALIASES.into_iter().flat_map(|(shape, names)| names.iter().map(move |name| (shape, *name)));
    for (shape, name) in aliases.chain(legacy) {
        symlink(shape.name(), cursors.join(name))?;
    }
    Ok(())
}
