//! Preview GIFs: a tour that chains transitions and idle loops, sheets of
//! every transition and every idle loop, and a few cursors in every bundled
//! palette.

use std::fs::{self, File};
use std::io;
use std::path::Path;

use tiny_skia::{Pixmap, PixmapPaint, Transform};

use crate::palette::Palette;
use crate::shape::{PAIRS, Shape};
use crate::theme::{Cursors, Fit, Fitted, Sequence, Theme, cursor_sequence, fitted, parallel, pixmaps};

/// Nominal size of sheet cells, and of the tour.
const SIZE: u32 = 64;
const TOUR_SIZE: u32 = 96;
/// Centiseconds per slowed transition frame, and held at either end.
const STEP: u32 = 2;
const HOLD: u32 = 40;
/// Shortest idle stop on the tour; it then plays on until back at rest.
const STOP: u32 = 150;
/// Sheet columns.
const COLUMNS: u32 = 5;
/// Cursors shown in each palette, one row per palette.
const SWATCHES: [Shape; 6] = [Shape::Default, Shape::Pointer, Shape::Text, Shape::Grab, Shape::NotAllowed, Shape::Wait];

/// Tour stops, looping back to the first. Each stop and the next are a pair
/// in `PAIRS`; a cursor idles on its first visit and passes through after.
pub const TOUR: [Shape; 12] = [
    Shape::Default,
    Shape::Pointer,
    Shape::Text,
    Shape::EwResize,
    Shape::Text,
    Shape::Default,
    Shape::Grab,
    Shape::Grabbing,
    Shape::Copy,
    Shape::Grabbing,
    Shape::Default,
    Shape::Wait,
];

type Images = Vec<(Pixmap, (u32, u32))>;
/// Frames to show, by index, with centiseconds each.
type Steps = Vec<(usize, u32)>;

/// Frames with their hotspots, the time in centiseconds each starts showing,
/// and the length of the whole clip.
struct Clip {
    images: Images,
    starts: Vec<(u32, usize)>,
    length: u32,
}

impl Clip {
    /// Shows each `(frame, centiseconds)` step in turn.
    fn new(images: Images, steps: impl IntoIterator<Item = (usize, u32)>) -> Clip {
        let mut length = 0;
        let mut starts = Vec::new();
        for (index, delay) in steps {
            starts.push((length, index));
            length += delay;
        }
        Clip { images, starts, length }
    }
}

fn centis(delay: u32) -> u32 {
    (delay / 10).max(2)
}

/// An idle loop from its start until it comes back to its first frame after
/// at least `least` centiseconds and, if `every`, after showing every stored
/// frame; or for `least` if it never moves.
fn idle<S>(images: &[(Pixmap, (u32, u32))], sequence: &Sequence<S>, least: u32, every: bool) -> Steps {
    let rest = &images[sequence.order[0]].0;
    let (mut time, mut unseen) = (0, if every { images.len() } else { 0 });
    let mut seen = vec![false; images.len()];
    let steps = sequence.order.iter().enumerate().map_while(|(n, &index)| {
        let back = n > 0 && time >= least && unseen == 0 && images[index].0 == *rest;
        if every && !std::mem::replace(&mut seen[index], true) {
            unseen -= 1;
        }
        let delay = if sequence.order.len() == 1 { least } else { centis(sequence.frames[index].delay) };
        time += delay;
        (!back).then_some((index, delay))
    });
    steps.collect()
}

/// Index in `PAIRS` of the transition between two cursors, and whether it
/// plays backwards.
fn between(from: Shape, to: Shape) -> (usize, bool) {
    match PAIRS.iter().position(|&pair| pair == (from, to)) {
        Some(index) => (index, false),
        None => (PAIRS.iter().position(|&pair| pair == (to, from)).expect("tour stops are pairs"), true),
    }
}

/// Writes `tour.gif`, `transitions.gif`, `idle.gif` and `palettes.gif`.
pub fn previews<T: Theme>(theme: &T, directory: &Path) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    let Fitted { shapes, transitions, fit } = fitted(theme);
    tour(theme, &shapes, &transitions, fit, &directory.join("tour.gif"))?;

    let clips = parallel(&transitions, |sequence| {
        let last = sequence.frames.len() - 1;
        let hold = |index: usize, end: usize| if index == end { HOLD } else { STEP };
        let steps = (0..=last).map(|i| (i, hold(i, last))).chain((0..=last).rev().map(|i| (i, hold(i, 0))));
        Clip::new(pixmaps(theme, sequence, fit, SIZE), steps)
    });
    compose(&clips, COLUMNS, SIZE * 3 / 2, &directory.join("transitions.gif"))?;

    let loops = parallel(&Shape::ALL, |&shape| {
        let images = pixmaps(theme, &shapes[&shape], fit, SIZE);
        let steps = idle(&images, &shapes[&shape], 1, true);
        (shape, images, steps)
    });
    compose(&synchronized(loops), COLUMNS, SIZE * 3 / 2, &directory.join("idle.gif"))?;

    // One gesture each, so long shuffled loops stay short.
    let swatches: Vec<(&str, Shape)> =
        T::PALETTES.iter().flat_map(|&(name, _)| SWATCHES.map(|shape| (name, shape))).collect();
    let loops = parallel(&swatches, |&(name, shape)| {
        let theme = T::new(Palette::load::<T>(name).expect("bundled palettes parse"));
        let sequence = cursor_sequence(&theme, shape);
        let images = pixmaps(&theme, &sequence, fit, SIZE);
        let steps = idle(&images, &sequence, 1, false);
        (shape, images, steps)
    });
    compose(&synchronized(loops), SWATCHES.len() as u32, SIZE * 3 / 2, &directory.join("palettes.gif"))
}

fn tour<T: Theme>(
    theme: &T,
    shapes: &Cursors<T::Scene>,
    transitions: &[Sequence<T::Scene>],
    fit: Fit,
    path: &Path,
) -> io::Result<()> {
    let legs: Vec<(Shape, Shape)> = TOUR.iter().zip(TOUR.iter().cycle().skip(1)).map(|(&a, &b)| (a, b)).collect();
    let stops = parallel(&TOUR, |shape| pixmaps(theme, &shapes[shape], fit, TOUR_SIZE));
    let moves = parallel(&legs, |&(from, to)| {
        let (index, reversed) = between(from, to);
        let mut images = pixmaps(theme, &transitions[index], fit, TOUR_SIZE);
        if reversed {
            images.reverse();
        }
        images
    });
    let (mut images, mut steps) = (Images::new(), Vec::new());
    for (n, (shape, (stop, leg))) in TOUR.iter().zip(stops.into_iter().zip(moves)).enumerate() {
        let offset = images.len();
        let stay = match TOUR[..n].contains(shape) {
            false => idle(&stop, &shapes[shape], STOP, false),
            true => vec![(shapes[shape].order[0], HOLD)],
        };
        steps.extend(stay.into_iter().map(|(index, delay)| (offset + index, delay)));
        images.extend(stop);
        let offset = images.len();
        steps.extend((1..leg.len() - 1).map(|index| (offset + index, STEP)));
        images.extend(leg);
    }
    compose(&[Clip::new(images, steps)], 1, TOUR_SIZE * 2, path)
}

/// Loops that all restart together: each plays as many whole times as fit in
/// the longest, then rests on its first frame. Busy loops never rest, so they
/// stretch to fill it instead.
fn synchronized(loops: Vec<(Shape, Images, Steps)>) -> Vec<Clip> {
    let period = |steps: &[(usize, u32)]| if steps.len() > 1 { steps.iter().map(|(_, delay)| delay).sum() } else { 0 };
    let longest = loops.iter().map(|(_, _, steps)| period(steps)).max().unwrap_or(0).max(HOLD);
    loops
        .into_iter()
        .map(|(shape, images, steps)| {
            let span = period(&steps);
            let (loops, stretch) = match span {
                0 => (1, 1.0),
                _ if shape.is_busy() => {
                    let loops = (longest as f32 / span as f32).round() as u32;
                    (loops, longest as f32 / (loops * span) as f32)
                }
                _ => (longest / span, 1.0),
            };
            let once: Vec<(u32, usize)> = Clip::new(Vec::new(), steps.iter().copied()).starts;
            let played = (0..loops).flat_map(|n| once.iter().map(move |&(time, index)| (n * span + time, index)));
            let at = |(time, index): (u32, usize)| ((time as f32 * stretch).round() as u32, index);
            let rest = (span > 0 && !shape.is_busy()).then(|| (loops * span, steps[0].0));
            Clip { images, starts: played.chain(rest).map(at).collect(), length: longest }
        })
        .collect()
}

/// Lays clips out in a grid of `side` square cells and writes them as one
/// GIF. Each clip keeps its hotspot still, placed so its frames center on
/// the cell, and holds its last frame until the longest ends.
fn compose(clips: &[Clip], columns: u32, side: u32, path: &Path) -> io::Result<()> {
    let length = clips.iter().map(|clip| clip.length).max().unwrap_or(0);
    let origins: Vec<Vec<(i32, i32)>> = clips
        .iter()
        .map(|clip| {
            let hots = clip.images.iter().map(|(_, hot)| *hot);
            let (x0, y0, x1, y1) = hots.fold((u32::MAX, u32::MAX, 0, 0), |(x0, y0, x1, y1), (x, y)| {
                (x0.min(x), y0.min(y), x1.max(x), y1.max(y))
            });
            let (mx, my) = ((x0 + x1) as i32 / 2, (y0 + y1) as i32 / 2);
            let corner = |size: u32| (side as i32 - size as i32) / 2;
            let place = |(image, (x, y)): &(Pixmap, (u32, u32))| {
                (corner(image.width()) + mx - *x as i32, corner(image.height()) + my - *y as i32)
            };
            clip.images.iter().map(place).collect()
        })
        .collect();
    // Browsers slow down delays under 2 cs, so changes closer than that wait.
    let mut times: Vec<u32> = clips.iter().flat_map(|clip| clip.starts.iter().map(|&(time, _)| time)).collect();
    times.sort_unstable();
    let times = times.into_iter().fold(vec![0], |mut kept, time| {
        if time >= kept[kept.len() - 1] + 2 && time + 2 <= length {
            kept.push(time);
        }
        kept
    });
    let rows = (clips.len() as u32).div_ceil(columns);
    let frames = times.iter().zip(times.iter().skip(1).chain([&length])).map(|(&time, &next)| {
        let mut canvas = Pixmap::new(side * columns, side * rows).expect("nonzero size");
        for (cell, (clip, origins)) in clips.iter().zip(&origins).enumerate() {
            let index = clip.starts.iter().take_while(|(start, _)| *start <= time).last().map_or(0, |&(_, i)| i);
            let mut tile = Pixmap::new(side, side).expect("nonzero size");
            let (x, y) = origins[index];
            tile.draw_pixmap(x, y, clip.images[index].0.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            let (x, y) = (cell as u32 % columns * side, cell as u32 / columns * side);
            canvas.draw_pixmap(x as i32, y as i32, tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
        }
        (canvas, (next - time) as u16)
    });
    gif(path, frames)
}

/// Pixel box as (left, top, right, bottom), inclusive.
type Area = (usize, usize, usize, usize);

/// Writes frames as a looping GIF over a neutral grey; delays are in
/// hundredths of a second. Each frame after the first stores only the box
/// that changed, and frames that change nothing extend the previous one.
fn gif(path: &Path, frames: impl IntoIterator<Item = (Pixmap, u16)>) -> io::Result<()> {
    const BACKGROUND: [u8; 3] = [0x55, 0x55, 0x55];
    let flatten = |pixmap: &Pixmap| -> Vec<u8> {
        pixmap
            .pixels()
            .iter()
            .flat_map(|pixel| {
                let alpha = 255 - pixel.alpha() as u16;
                let channels = [pixel.red(), pixel.green(), pixel.blue()];
                (0..3).map(move |c| (channels[c] as u16 + BACKGROUND[c] as u16 * alpha / 255) as u8)
            })
            .collect()
    };
    let mut frames = frames.into_iter().peekable();
    let (width, height) =
        frames.peek().map_or((1, 1), |(pixmap, _)| (pixmap.width() as usize, pixmap.height() as usize));
    let changed = |a: &[u8], b: &[u8]| -> Option<Area> {
        (0..width * height).filter(|&i| a[3 * i..3 * i + 3] != b[3 * i..3 * i + 3]).fold(None, |area, i| {
            let (x, y) = (i % width, i / width);
            let (l, t, r, b) = area.unwrap_or((x, y, x, y));
            Some((l.min(x), t.min(y), r.max(x), b.max(y)))
        })
    };
    let mut encoder =
        gif::Encoder::new(File::create(path)?, width as u16, height as u16, &[]).map_err(io::Error::other)?;
    encoder.set_repeat(gif::Repeat::Infinite).map_err(io::Error::other)?;
    let mut write = |(rgb, (l, t, r, b), delay): (Vec<u8>, Area, u16)| {
        let mut frame = gif::Frame::from_rgb_speed((r - l + 1) as u16, (b - t + 1) as u16, &rgb, 10);
        (frame.left, frame.top, frame.delay) = (l as u16, t as u16, delay);
        encoder.write_frame(&frame).map_err(io::Error::other)
    };
    let mut pending: Option<(Vec<u8>, Area, u16)> = None;
    let mut previous: Option<Vec<u8>> = None;
    for (pixmap, delay) in frames {
        let rgb = flatten(&pixmap);
        let area = match &previous {
            None => Some((0, 0, width - 1, height - 1)),
            Some(last) => changed(last, &rgb),
        };
        match (area, &mut pending) {
            (None, Some(last)) => last.2 += delay,
            (area, _) => {
                let (l, t, r, b) = area.unwrap_or((0, 0, 0, 0));
                let crop = (t..=b).flat_map(|y| rgb[3 * (y * width + l)..3 * (y * width + r + 1)].to_vec()).collect();
                if let Some(done) = pending.replace((crop, (l, t, r, b), delay)) {
                    write(done)?;
                }
                previous = Some(rgb);
            }
        }
    }
    pending.map_or(Ok(()), write)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tour_legs_are_pairs() {
        for (&from, &to) in TOUR.iter().zip(TOUR.iter().cycle().skip(1)) {
            assert!(PAIRS.contains(&(from, to)) || PAIRS.contains(&(to, from)), "{from:?} to {to:?}");
        }
    }
}
