//! Each cursor as a scene of M3 shapes and white glyphs, in design units
//! with the hotspot at the origin.

use cursor_core::color::{Color, mix as mix_color};
use cursor_core::motion::{bump, ease, polar, spring, window, wobble};
use cursor_core::{Palette, Shape};

use crate::outline::{Outline, P, circle, cookie, custom, flower, pentagon, pill, polygon, star, stretch, sunny};
use crate::scene::{Blob, Glyph, Scene};

fn blob(center: P, outline: Outline, rotation: f32, color: Color) -> Blob {
    Blob { center, outline, rotation, color }
}

fn stroke(key: &'static str, points: Vec<P>, width: f32) -> Glyph {
    let n = points.len() as f32;
    let pivot = points.iter().fold((0.0, 0.0), |(x, y), p| (x + p.0 / n, y + p.1 / n));
    Glyph { key, points, width, scale: 1.0, pivot }
}

fn dot(key: &'static str, (x, y): P, diameter: f32) -> Glyph {
    stroke(key, vec![(x, y), (x + 0.001, y)], diameter)
}

fn chevron(key: &'static str, tip: P, angle: f32, arm: f32) -> Glyph {
    let ends = [45.0, -45.0].map(|turn| polar(tip.0, tip.1, arm, angle + 180.0 + turn));
    stroke(key, vec![ends[0], tip, ends[1]], 1.9)
}

/// Rounded arrow with its tip on the hotspot.
/// Center of the arrow body. Every other shape sits here too, so morphs from
/// the arrow stay in place and the hotspot stays at the arrow tip.
const ANCHOR: P = (7.2, 7.2);

fn arrow(color: Color) -> Blob {
    let center = ANCHOR;
    let corners = [((0.0, 0.0), 1.0), ((15.5, 6.0), 3.0), ((6.0, 15.5), 3.0)];
    let local: Vec<(P, f32)> = corners.iter().map(|&((x, y), r)| ((x - center.0, y - center.1), r)).collect();
    blob(center, custom(&local), 0.0, color)
}

/// Pill along `angle` with chevrons pointing out of both ends.
fn resize(colors: &Palette, angle: f32) -> Scene {
    let glyphs = [("arrow0", 0.0), ("arrow1", 180.0)]
        .map(|(key, turn)| chevron(key, polar(0.0, 0.0, 7.2, angle + turn), angle + turn, 2.6));
    Scene { blobs: vec![blob((0.0, 0.0), pill(22.0, 10.0, 0.0), angle, colors["resize"])], glyphs: glyphs.into() }
}

/// A question mark: a hook curling into a stem, and a dot below.
fn query() -> Vec<Glyph> {
    let mut hook: Vec<P> = (0..=10).map(|i| polar(0.0, -2.2, 2.5, 200.0 + 25.0 * i as f32)).collect();
    hook.push((0.0, 1.4));
    vec![stroke("hook", hook, 2.2), dot("dot", (0.0, 4.3), 2.7)]
}

fn plus(arm: f32) -> Vec<Glyph> {
    vec![stroke("bar", vec![(-arm, 0.0), (arm, 0.0)], 2.2), stroke("stem", vec![(0.0, -arm), (0.0, arm)], 2.2)]
}

/// Petals spread open, ready to grab.
fn bloom(colors: &Palette) -> Blob {
    blob((0.0, 0.0), star(8, 10.4, 0.55, 10.0, 0.6, 0.0), 0.0, colors["grab"])
}

/// The bloom closed tight around what it holds.
fn fist(color: Color) -> Blob {
    blob((0.0, 0.0), cookie(8, 7.6), 22.5, color)
}

const LOADING: usize = 7;
const LOAD: [&str; LOADING] = ["load0", "load1", "load2", "load3", "load4", "load5", "load6"];

/// One step of the loading sequence: simple rounded shapes, each in its own
/// color.
fn loading(colors: &Palette, step: usize, r: f32) -> (Outline, Color) {
    let outline = match step % LOADING {
        0 => polygon(4, r, 0.22 * r, 45.0),
        1 => polygon(3, r, 0.22 * r, 0.0),
        2 => polygon(6, r, 0.2 * r, 0.0),
        3 => pill(2.0 * r, 1.1 * r, 0.0),
        4 => pentagon(r),
        5 => stretch(&polygon(4, r, 0.2 * r, 0.0), 0.8, 1.1),
        _ => circle(0.85 * r),
    };
    (outline, colors[LOAD[step % LOADING]])
}

/// The loading indicator at loop phase `u`, centered on `center`: it springs
/// from shape to shape, sweeping to each shape's color, while spinning one
/// turn per loop.
fn indicator(colors: &Palette, u: f32, r: f32, center: P) -> Blob {
    let position = u * LOADING as f32;
    let step = position.floor() as usize;
    let progress = window(position.fract(), (0.0, 0.6));
    let ((a, from), (b, to)) = (loading(colors, step, r), loading(colors, step + 1, r));
    let color = mix_color(from, to, ease(progress));
    Blob { center, outline: a.mix(&b, spring(progress)), rotation: 360.0 * u, color }
}

/// Moves a layout into place: the arrows already sit at the tip, every other
/// shape moves to the anchor.
fn place(shape: Shape, scene: Scene) -> Scene {
    if matches!(shape, Shape::Default | Shape::Pointer | Shape::Progress) { scene } else { scene.shift(ANCHOR) }
}

pub fn scene(colors: &Palette, shape: Shape, phase: f32) -> Scene {
    place(shape, layout(colors, shape, phase))
}

const BUSY_FRAMES: usize = 84;
const BUSY_DELAY: u32 = 60;
/// Idle gesture frame length in milliseconds.
const STEP: u32 = 30;

/// Milliseconds at rest, then gesture frames.
fn timing(shape: Shape) -> (u32, usize) {
    match shape {
        Shape::Default => (1600, 22),
        Shape::Text | Shape::VerticalText => (1800, 18),
        Shape::Pointer | Shape::Grab | Shape::Copy | Shape::ColResize => (1200, 24),
        Shape::NotAllowed | Shape::NoDrop => (1000, 20),
        _ => (1400, 20),
    }
}

/// The idle gesture of `rest`, the unplaced resting layout, at progress `u`
/// in (0, 1). Spins turn by a symmetry of the shape, so the loop closes
/// seamlessly.
fn gesture(shape: Shape, rest: Scene, u: f32) -> Scene {
    let origin = (0.0, 0.0);
    match shape {
        Shape::Default => rest.about(origin, 9.0 * wobble(u, 1.5), 1.0),
        Shape::Pointer => {
            rest.about(origin, 0.0, 1.0 - 0.06 * bump(u, 0.0, 0.5)).pulse_glyphs(1.0 + 0.45 * bump(u, 0.1, 0.6))
        }
        Shape::Text | Shape::VerticalText => rest.about(origin, 0.0, 1.0 + 0.08 * wobble(u, 1.0)),
        Shape::EwResize | Shape::NsResize | Shape::NeswResize | Shape::NwseResize => {
            rest.push_glyphs(1.3 * (bump(u, 0.0, 0.45) + bump(u, 0.5, 0.45)))
        }
        Shape::ColResize => rest.spin(90.0 * spring(u)).push_glyphs(1.0 * bump(u, 0.15, 0.6)),
        Shape::Grab => rest.spin(45.0 * spring(u)).about(origin, 0.0, 1.0 + 0.05 * bump(u, 0.0, 0.7)),
        Shape::Grabbing => rest.about(origin, 0.0, 1.0 - 0.08 * (bump(u, 0.0, 0.45) + bump(u, 0.5, 0.45))),
        Shape::Copy => rest.about(origin, 90.0 * spring(u), 1.0),
        Shape::NoDrop => rest.shift((1.3 * (std::f32::consts::TAU * 3.0 * u).sin() * (1.0 - u), 0.0)),
        Shape::NotAllowed => rest.about(origin, 10.0 * wobble(u, 2.0), 1.0),
        Shape::Crosshair => rest.push_blobs(-1.4 * bump(u, 0.0, 1.0)),
        Shape::ZoomIn => rest.spin(45.0 * spring(u)),
        Shape::ZoomOut => rest.spin(30.0 * spring(u)),
        Shape::Help => rest.spin(40.0 * spring(u)),
        Shape::Kill => rest.spin(90.0 * spring(u)).about(origin, 0.0, 1.0 + 0.06 * bump(u, 0.0, 0.5)),
        Shape::Progress | Shape::Wait => rest,
    }
}

/// Frames of an ordinary cursor: a busy loop, or rest then an idle gesture.
pub fn frames(colors: &Palette, shape: Shape) -> Vec<(Scene, u32)> {
    if shape.is_busy() {
        return (0..BUSY_FRAMES).map(|i| (scene(colors, shape, i as f32 / BUSY_FRAMES as f32), BUSY_DELAY)).collect();
    }
    let (hold, count) = timing(shape);
    let rest = layout(colors, shape, 0.0);
    std::iter::once((place(shape, rest.clone()), hold))
        .chain((1..count).map(|i| (place(shape, gesture(shape, rest.clone(), i as f32 / count as f32)), STEP)))
        .collect()
}

/// Shapes around the origin; the arrows place themselves.
fn layout(colors: &Palette, shape: Shape, phase: f32) -> Scene {
    match shape {
        Shape::Default => Scene { blobs: vec![arrow(colors["arrow"])], glyphs: vec![] },
        Shape::Pointer => Scene { blobs: vec![arrow(colors["link"])], glyphs: vec![dot("point", (5.6, 5.6), 2.6)] },
        Shape::Progress => {
            Scene { blobs: vec![arrow(colors["arrow"]), indicator(colors, phase, 3.8, (14.0, 14.0))], glyphs: vec![] }
        }
        Shape::Wait => Scene { blobs: vec![indicator(colors, phase, 9.5, (0.0, 0.0))], glyphs: vec![] },
        Shape::Text => {
            Scene { blobs: vec![blob((0.0, 0.0), pill(20.0, 4.2, 0.0), 90.0, colors["text"])], glyphs: vec![] }
        }
        Shape::VerticalText => {
            Scene { blobs: vec![blob((0.0, 0.0), pill(20.0, 4.2, 0.0), 0.0, colors["text"])], glyphs: vec![] }
        }
        Shape::EwResize => resize(colors, 0.0),
        Shape::NsResize => resize(colors, 90.0),
        Shape::NeswResize => resize(colors, -45.0),
        Shape::NwseResize => resize(colors, 45.0),
        Shape::ColResize => {
            let mut glyphs = vec![stroke("bar", vec![(0.0, -4.6), (0.0, 4.6)], 1.9)];
            glyphs.extend(
                [("arrow0", 0.0), ("arrow1", 180.0)]
                    .map(|(key, angle)| chevron(key, polar(0.0, 0.0, 6.0, angle), angle, 2.4)),
            );
            Scene { blobs: vec![blob((0.0, 0.0), cookie(4, 10.5), 45.0, colors["resize"])], glyphs }
        }
        Shape::Grab => Scene { blobs: vec![bloom(colors)], glyphs: vec![dot("dot", (0.0, 0.0), 3.0)] },
        Shape::Grabbing => Scene { blobs: vec![fist(colors["grabbing"])], glyphs: vec![dot("dot", (0.0, 0.0), 3.6)] },
        Shape::Copy => Scene { blobs: vec![blob((0.0, 0.0), flower(10.0), 0.0, colors["copy"])], glyphs: plus(3.6) },
        Shape::NoDrop => {
            let ring = (0..=24).map(|i| polar(0.0, 0.0, 3.6, 15.0 * i as f32)).collect();
            Scene {
                blobs: vec![fist(colors["refuse"])],
                glyphs: vec![stroke("ring", ring, 1.9), stroke("slash", vec![(-2.5, -2.5), (2.5, 2.5)], 1.9)],
            }
        }
        Shape::NotAllowed => Scene {
            blobs: vec![blob((0.0, 0.0), polygon(8, 10.2, 2.2, 22.5), 0.0, colors["refuse"])],
            glyphs: vec![stroke("bar", vec![(-4.4, 0.0), (4.4, 0.0)], 2.8)],
        },
        Shape::Crosshair => {
            let mut blobs = vec![blob((0.0, 0.0), circle(1.8), 0.0, colors["crosshair"])];
            blobs.extend([(-90.0, 180.0), (0.0, -90.0), (90.0, 0.0), (180.0, 90.0)].map(|(at, turn)| {
                blob(polar(0.0, 0.0, 8.0, at), polygon(3, 2.8, 0.7, turn), 0.0, colors["crosshair"])
            }));
            Scene { blobs, glyphs: vec![] }
        }
        Shape::Help => Scene { blobs: vec![blob((0.0, 0.0), cookie(9, 10.2), 0.0, colors["help"])], glyphs: query() },
        Shape::Kill => Scene {
            blobs: vec![blob((0.0, 0.0), star(4, 10.8, 0.32, 2.4, 1.0, 45.0), 0.0, colors["refuse"])],
            glyphs: vec![dot("dot", (0.0, 0.0), 3.0)],
        },
        Shape::ZoomIn => Scene { blobs: vec![blob((0.0, 0.0), sunny(10.0), 0.0, colors["zoom"])], glyphs: plus(3.6) },
        Shape::ZoomOut => Scene {
            blobs: vec![blob((0.0, 0.0), cookie(12, 9.6), 0.0, colors["zoom"])],
            glyphs: vec![plus(3.6).remove(0)],
        },
    }
}
