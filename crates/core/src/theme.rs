//! The theme interface and the writer for a complete Xcursor theme:
//! ordinary cursors, reversible transitions, aliases and preview GIFs.

use std::fs::{self, File};
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use tiny_skia::{Pixmap, Transform};

use crate::paint::argb;
use crate::shape::{LEGACY_ALIASES, PAIRS, Shape};
use crate::xcursor::{Image, encode};

/// Design units per nominal cursor size.
pub const GRID: f32 = 32.0;
pub const SIZES: [u32; 6] = [24, 32, 40, 48, 56, 64];
pub const TRANSITION_FRAMES: usize = 24;
pub const TRANSITION_DELAY: u32 = 5;

pub struct Frame<S> {
    pub scene: S,
    /// Milliseconds; 0 for a static cursor.
    pub delay: u32,
}

pub trait Theme: Sync {
    /// Whatever the theme needs to draw one frame.
    type Scene: Sync;

    /// Installed directory name.
    const ID: &'static str;
    const TITLE: &'static str;

    /// Hotspot position inside the 32x32 cell of an ordinary cursor. Multiples
    /// of 4 keep it on a whole pixel at every size.
    fn hotspot(&self, shape: Shape) -> (f32, f32);

    /// Frames of an ordinary cursor; more than one makes it loop.
    fn cursor(&self, shape: Shape) -> Vec<Frame<Self::Scene>>;

    /// The scene at progress `t` in (0, 1) of the transition from `from` to
    /// `to`. Both endpoints are the cursors' first frames. Playing it
    /// backwards should match the transition from `to` to `from`.
    fn transition(&self, from: Shape, to: Shape, t: f32) -> Self::Scene;

    /// Draws a scene in design units with the hotspot at the origin.
    fn draw(&self, scene: &Self::Scene, pixmap: &mut Pixmap, transform: Transform);
}

fn first<T: Theme>(theme: &T, shape: Shape) -> T::Scene {
    theme.cursor(shape).into_iter().next().expect("cursor has frames").scene
}

pub fn transition_frames<T: Theme>(theme: &T, from: Shape, to: Shape) -> Vec<Frame<T::Scene>> {
    let last = TRANSITION_FRAMES - 1;
    (0..TRANSITION_FRAMES)
        .map(|index| match index {
            0 => first(theme, from),
            _ if index == last => first(theme, to),
            _ => theme.transition(from, to, index as f32 / last as f32),
        })
        .map(|scene| Frame { scene, delay: TRANSITION_DELAY })
        .collect()
}

/// Renders a scene on a `canvas` square with the hotspot at the center of
/// pixel `hotspot`.
pub fn render<T: Theme>(theme: &T, scene: &T::Scene, size: u32, canvas: u32, hotspot: (u32, u32)) -> Pixmap {
    let scale = size as f32 / GRID;
    let (x, y) = (hotspot.0 as f32 + 0.5, hotspot.1 as f32 + 0.5);
    let mut pixmap = Pixmap::new(canvas, canvas).expect("nonzero size");
    theme.draw(scene, &mut pixmap, Transform::from_row(scale, 0.0, 0.0, scale, x, y));
    pixmap
}

/// Hotspot pixel of an ordinary cursor at `size`.
pub fn hotspot_pixel<T: Theme>(theme: &T, shape: Shape, size: u32) -> (u32, u32) {
    let (x, y) = theme.hotspot(shape);
    let pixel = |unit: f32| (unit * size as f32 / GRID).round() as u32;
    (pixel(x), pixel(y))
}

/// Ordinary cursors fill one nominal-size square; transitions get twice that,
/// centered on the hotspot, to fit both endpoints.
pub fn images<T: Theme>(theme: &T, frames: &[Frame<T::Scene>], size: u32, layout: Layout) -> Vec<Image> {
    let cursor = |shape: Shape, scene: &T::Scene| {
        let hotspot = hotspot_pixel(theme, shape, size);
        (render(theme, scene, size, size, hotspot), hotspot)
    };
    let last = frames.len() - 1;
    frames
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            let (pixmap, (xhot, yhot)) = match layout {
                Layout::Cursor(shape) => cursor(shape, &frame.scene),
                Layout::Transition(from, to) if index == 0 || index == last => {
                    let (pixmap, hotspot) = cursor(if index == 0 { from } else { to }, &frame.scene);
                    (centered(&pixmap, hotspot, size), (size, size))
                }
                Layout::Transition(..) => (render(theme, &frame.scene, size, size * 2, (size, size)), (size, size)),
            };
            let (width, height) = (pixmap.width(), pixmap.height());
            Image { size, width, height, xhot, yhot, delay: frame.delay, pixels: argb(&pixmap) }
        })
        .collect()
}

/// Copies an ordinary cursor onto a transition canvas, hotspot at its center,
/// so endpoints match the cursor exactly.
fn centered(pixmap: &Pixmap, (xhot, yhot): (u32, u32), size: u32) -> Pixmap {
    let mut canvas = Pixmap::new(size * 2, size * 2).expect("nonzero size");
    let (dx, dy) = ((size - xhot) as usize, (size - yhot) as usize);
    let (width, stride) = (pixmap.width() as usize * 4, size as usize * 8);
    for (row, line) in pixmap.data().chunks_exact(width).enumerate() {
        let start = (row + dy) * stride + dx * 4;
        canvas.data_mut()[start..start + width].copy_from_slice(line);
    }
    canvas
}

#[derive(Clone, Copy, Debug)]
pub enum Layout {
    Cursor(Shape),
    Transition(Shape, Shape),
}

/// Every name for a transition, canonical first.
pub fn transition_names(from: Shape, to: Shape) -> Vec<String> {
    let names = |shape: Shape| std::iter::once(shape.name()).chain(shape.aliases().iter().copied());
    names(from).flat_map(|a| names(to).map(move |b| format!("{a}-to-{b}"))).collect()
}

/// Loops play in real time; transitions play slowed down, both ways, with
/// pauses at the endpoints.
fn preview<T: Theme>(theme: &T, frames: &[Frame<T::Scene>], transition: bool, path: &Path) -> io::Result<()> {
    const SIZE: u32 = 64;
    let canvas = SIZE * 2;
    let pixmaps: Vec<Pixmap> =
        frames.iter().map(|frame| render(theme, &frame.scene, SIZE, canvas, (SIZE, SIZE))).collect();
    let last = pixmaps.len() - 1;
    let sequence: Vec<(usize, u16)> = if transition {
        let hold = |index: usize, end: usize| if index == end { 40 } else { 2 };
        (0..=last).map(|i| (i, hold(i, last))).chain((0..=last).rev().map(|i| (i, hold(i, 0)))).collect()
    } else {
        frames.iter().enumerate().map(|(i, frame)| (i, (frame.delay / 10).max(2) as u16)).collect()
    };
    let (frames, delays): (Vec<&Pixmap>, Vec<u16>) =
        sequence.into_iter().map(|(i, delay)| (&pixmaps[i], delay)).unzip();
    gif(path, &frames, &delays, 2)
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

fn parallel<J: Sync>(jobs: &[J], work: impl Fn(&J) -> io::Result<()> + Sync) -> io::Result<()> {
    let next = AtomicUsize::new(0);
    let workers = thread::available_parallelism().map_or(4, |n| n.get());
    thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    while let Some(job) = jobs.get(next.fetch_add(1, Ordering::Relaxed)) {
                        work(job)?;
                    }
                    Ok(())
                })
            })
            .collect();
        handles.into_iter().try_for_each(|handle| handle.join().expect("worker panicked"))
    })
}

pub fn generate<T: Theme>(theme: &T, output: &Path, previews: Option<&Path>) -> io::Result<()> {
    let cursors = output.join("cursors");
    fs::create_dir_all(&cursors)?;
    if let Some(directory) = previews {
        fs::create_dir_all(directory)?;
    }
    fs::write(output.join("index.theme"), format!("[Icon Theme]\nName={}\n", T::TITLE))?;
    let jobs: Vec<(String, Layout)> = Shape::ALL
        .into_iter()
        .map(|shape| (shape.name().to_owned(), Layout::Cursor(shape)))
        .chain(PAIRS.into_iter().map(|(a, b)| (format!("{}-to-{}", a.name(), b.name()), Layout::Transition(a, b))))
        .collect();
    parallel(&jobs, |(name, layout)| {
        let frames = match layout {
            Layout::Cursor(shape) => theme.cursor(*shape),
            Layout::Transition(a, b) => transition_frames(theme, *a, *b),
        };
        let images: Vec<Image> = SIZES.into_iter().flat_map(|size| images(theme, &frames, size, *layout)).collect();
        fs::write(cursors.join(name), encode(&images))?;
        let transition = matches!(layout, Layout::Transition(..));
        match previews {
            Some(directory) => preview(theme, &frames, transition, &directory.join(format!("{name}.gif"))),
            _ => Ok(()),
        }
    })?;
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
