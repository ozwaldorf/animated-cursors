//! Writes the cursor theme: ordinary cursors, transitions and aliases.

use std::fs::{self, File};
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use tiny_skia::Pixmap;

use crate::render::{argb, render};
use crate::rig::{Parts, Shape, blend};
use crate::xcursor::{Image, encode};

pub const SIZES: [u32; 6] = [24, 32, 40, 48, 56, 64];
pub const FRAMES: usize = 24;
pub const DELAY: u32 = 5;
pub const SPIN_FRAMES: usize = 36;
pub const SPIN_DELAY: u32 = 25;

pub const PAIRS: [(Shape, Shape); 19] = [
    (Shape::Default, Shape::Pointer),
    (Shape::Default, Shape::Text),
    (Shape::Pointer, Shape::Text),
    (Shape::Default, Shape::EwResize),
    (Shape::Default, Shape::NsResize),
    (Shape::Default, Shape::NeswResize),
    (Shape::Default, Shape::NwseResize),
    (Shape::Default, Shape::ColResize),
    (Shape::Default, Shape::Grab),
    (Shape::Grab, Shape::Grabbing),
    (Shape::Default, Shape::Grabbing),
    (Shape::Default, Shape::Progress),
    (Shape::Default, Shape::Wait),
    (Shape::Grabbing, Shape::Copy),
    (Shape::Grabbing, Shape::NotAllowed),
    (Shape::Grabbing, Shape::NoDrop),
    (Shape::Default, Shape::Crosshair),
    (Shape::Text, Shape::VerticalText),
    (Shape::ZoomIn, Shape::ZoomOut),
];

/// Legacy X11, Qt and toolkit names.
pub const LEGACY_ALIASES: [(Shape, &[&str]); 16] = [
    (
        Shape::Default,
        &[
            "arrow",
            "left_ptr",
            "top_left_arrow",
            "right_ptr",
            "center_ptr",
            "left_ptr_help",
            "question_arrow",
            "whats_this",
            "dnd-ask",
            "up-arrow",
            "down-arrow",
            "left-arrow",
            "right-arrow",
            "sb_up_arrow",
            "sb_down_arrow",
            "sb_left_arrow",
            "sb_right_arrow",
            "left_tee",
            "right_tee",
            "top_tee",
            "bottom_tee",
            "wayland-cursor",
            "5c6cd98b3f3ebcb1f9c7f1c204630408",
            "d9ce0ab605698f320427677b458ad60b",
        ],
    ),
    (
        Shape::Pointer,
        &[
            "hand",
            "hand1",
            "hand2",
            "pointing_hand",
            "9d800788f1b08800ae810202380a0822",
            "e29285e634086352946a0e7090d73106",
        ],
    ),
    (Shape::Text, &["xterm", "ibeam"]),
    (
        Shape::EwResize,
        &[
            "h_double_arrow",
            "sb_h_double_arrow",
            "size_hor",
            "size-hor",
            "left_side",
            "right_side",
            "028006030e0e7ebffc7f7070c0600140",
            "14fef782d02440884392942c11205230",
        ],
    ),
    (
        Shape::NsResize,
        &[
            "v_double_arrow",
            "sb_v_double_arrow",
            "double_arrow",
            "size_ver",
            "size-ver",
            "top_side",
            "bottom_side",
            "split_v",
            "00008160000006810000408080010102",
            "2870a09082c103050810ffdffffe0204",
        ],
    ),
    (
        Shape::NeswResize,
        &[
            "bd_double_arrow",
            "size_bdiag",
            "size-bdiag",
            "top_right_corner",
            "bottom_left_corner",
            "ur_angle",
            "ll_angle",
            "c7088f0f3e6c8088236ef8e1e3e70000",
        ],
    ),
    (
        Shape::NwseResize,
        &[
            "fd_double_arrow",
            "size_fdiag",
            "size-fdiag",
            "top_left_corner",
            "bottom_right_corner",
            "ul_angle",
            "lr_angle",
            "fcf1c3c7cd4491d801f1e1c78f100000",
        ],
    ),
    (Shape::ColResize, &["split_h"]),
    (Shape::Grab, &["openhand"]),
    (
        Shape::Grabbing,
        &[
            "closedhand",
            "dnd-move",
            "dnd-none",
            "fleur",
            "size_all",
            "4498f0e0c1937ffe01fd06f973665830",
            "9081237383d90e509aa00f00170e968f",
            "fcf21c00b30f7e3f83fe0dfd12e71cff",
        ],
    ),
    (
        Shape::Progress,
        &[
            "left_ptr_watch",
            "half-busy",
            "00000000000000020006000e7e9ffc3f",
            "08e8e1c95fe2fc01f976f1e063a24ccd",
            "3ecb610c1bf2410f44200f48c40d3599",
        ],
    ),
    (Shape::Wait, &["watch"]),
    (
        Shape::Copy,
        &[
            "dnd-copy",
            "link",
            "dnd-link",
            "1081e37283d90000800003c07f3ef6bf",
            "6407b0e94181790501fd1e167b474872",
            "b66166c04f8c3109214a4fbd64a50fc8",
            "3085a0e285430894940527032f8b26df",
            "640fb0e74195791501fd1ed57b41487f",
            "a2a266d0498c3104214a47bd64ab0fc8",
        ],
    ),
    (
        Shape::NotAllowed,
        &["circle", "crossed_circle", "pirate", "X_cursor", "x-cursor", "03b6e0fcb3499374a867c041f52298f0"],
    ),
    (Shape::NoDrop, &["dnd-no-drop", "forbidden"]),
    (
        Shape::Crosshair,
        &[
            "cross",
            "cross_reverse",
            "diamond_cross",
            "tcross",
            "plus",
            "color-picker",
            "target",
            "dotbox",
            "dot_box_mask",
            "draped_box",
            "icon",
            "pencil",
            "draft",
            "draft_large",
            "draft_small",
        ],
    ),
];

pub type Frames = Vec<(Parts, u32)>;

pub fn shape_frames(shape: Shape) -> Frames {
    if shape.is_spinner() {
        (0..SPIN_FRAMES).map(|index| (shape.parts(index as f32 / SPIN_FRAMES as f32), SPIN_DELAY)).collect()
    } else {
        vec![(shape.parts(0.0), 0)]
    }
}

/// Endpoints are the shapes' exact first frames, so a finished transition is
/// pixel-identical to the ordinary cursor.
pub fn transition_frames(from: Shape, to: Shape) -> Frames {
    let (a, b) = (from.parts(0.0), to.parts(0.0));
    (0..FRAMES)
        .map(|index| match index {
            0 => a.clone(),
            last if last == FRAMES - 1 => b.clone(),
            _ => blend(&a, &b, index as f32 / (FRAMES - 1) as f32),
        })
        .map(|parts| (parts, DELAY))
        .collect()
}

/// Every name for a transition, canonical first.
pub fn transition_names(from: Shape, to: Shape) -> Vec<String> {
    let names = |shape: Shape| std::iter::once(shape.name()).chain(shape.aliases().iter().copied());
    names(from).flat_map(|a| names(to).map(move |b| format!("{a}-to-{b}"))).collect()
}

pub fn images(frames: &Frames, size: u32) -> Vec<Image> {
    frames
        .iter()
        .map(|(parts, delay)| {
            let pixels = argb(&render(parts, size));
            Image { size, width: size, height: size, xhot: size / 2, yhot: size / 2, delay: *delay, pixels }
        })
        .collect()
}

fn preview(frames: &Frames, path: &Path) -> io::Result<()> {
    const SCALE: usize = 4;
    const BACKGROUND: [u8; 3] = [0x55, 0x55, 0x55];
    let size = *SIZES.last().unwrap();
    let side = size as usize * SCALE;
    let mut encoder =
        gif::Encoder::new(File::create(path)?, side as u16, side as u16, &[]).map_err(io::Error::other)?;
    encoder.set_repeat(gif::Repeat::Infinite).map_err(io::Error::other)?;
    let pixmaps: Vec<Pixmap> = frames.iter().map(|(parts, _)| render(parts, size)).collect();
    let last = pixmaps.len() - 1;
    let sequence = (0..=last).map(|i| (i, i == last)).chain((0..=last).rev().map(|i| (i, i == 0)));
    for (index, hold) in sequence {
        let rgb: Vec<u8> = (0..side * side)
            .flat_map(|i| {
                let pixel = pixmaps[index].pixel((i % side / SCALE) as u32, (i / side / SCALE) as u32).unwrap();
                let alpha = 255 - pixel.alpha() as u16;
                let channels = [pixel.red(), pixel.green(), pixel.blue()];
                (0..3).map(move |c| (channels[c] as u16 + BACKGROUND[c] as u16 * alpha / 255) as u8)
            })
            .collect();
        let mut frame = gif::Frame::from_rgb_speed(side as u16, side as u16, &rgb, 10);
        frame.delay = if hold { 40 } else { 2 };
        encoder.write_frame(&frame).map_err(io::Error::other)?;
    }
    Ok(())
}

fn parallel<T: Sync>(jobs: &[T], work: impl Fn(&T) -> io::Result<()> + Sync) -> io::Result<()> {
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

pub fn generate(output: &Path, previews: Option<&Path>) -> io::Result<()> {
    let cursors = output.join("cursors");
    fs::create_dir_all(&cursors)?;
    if let Some(directory) = previews {
        fs::create_dir_all(directory)?;
    }
    fs::write(
        output.join("index.theme"),
        "[Icon Theme]\nName=Animated Dot\nComment=Dot cursors with reversible transitions\n",
    )?;
    let jobs: Vec<(String, Frames, bool)> = Shape::ALL
        .into_iter()
        .map(|shape| (shape.name().to_owned(), shape_frames(shape), false))
        .chain(PAIRS.into_iter().map(|(a, b)| (format!("{}-to-{}", a.name(), b.name()), transition_frames(a, b), true)))
        .collect();
    parallel(&jobs, |(name, frames, transition)| {
        let images: Vec<Image> = SIZES.into_iter().flat_map(|size| images(frames, size)).collect();
        fs::write(cursors.join(name), encode(&images))?;
        match previews {
            Some(directory) if *transition => preview(frames, &directory.join(format!("{name}.gif"))),
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
