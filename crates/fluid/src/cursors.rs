//! Each cursor as a scene of liquid drops, in design units with the hotspot
//! at the origin.

use std::f32::consts::TAU;

use cursor_core::color::Color;
use cursor_core::motion::polar;
use cursor_core::{Palette, Shape};

use crate::idle;
use crate::scene::{Drop, Drops, P, Scene, blob, capsule, cone};

/// Teardrop with its tip on the hotspot, leaning like an arrow.
fn teardrop(color: Color) -> Drop {
    cone((0.7, 0.9), 0.9, (7.5, 10.5), 6.0, color)
}

/// Center of the teardrop's round end.
pub const BELLY: P = (7.5, 10.5);

fn rotated(drops: Drops, degrees: f32) -> Drops {
    let turn = |(x, y): P| polar(0.0, 0.0, x.hypot(y), y.atan2(x).to_degrees() + degrees);
    drops.into_iter().map(|(key, drop)| (key, drop.map(turn))).collect()
}

const ARROWS: [[&str; 2]; 2] = [["arrow0_l", "arrow0_r"], ["arrow1_l", "arrow1_r"]];

/// Two-armed ends along 0 and 180 degrees, tips `reach` from the origin, arms
/// turned `spread` degrees back from the axis: 135 makes arrowheads, 90 makes
/// serifs. Each arm starts at its tip, so serifs fold into arrowheads.
fn ends(reach: f32, arm: f32, spread: f32, color: Color) -> Drops {
    let mut drops = Drops::new();
    for (keys, angle) in ARROWS.into_iter().zip([0.0, 180.0]) {
        let tip = polar(0.0, 0.0, reach, angle);
        for (key, turn) in keys.into_iter().zip([spread, -spread]) {
            drops.push((key, capsule(tip, polar(tip.0, tip.1, arm, angle + turn), 1.15, color)));
        }
    }
    drops
}

/// Bar along `angle` with an arrowhead at each end.
fn resize(colors: &Palette, angle: f32) -> Drops {
    let color = colors["resize"];
    let mut drops = vec![("body", capsule((-8.5, 0.0), (8.5, 0.0), 1.15, color))];
    drops.extend(ends(9.5, 4.4, 135.0, color));
    rotated(drops, angle)
}

/// Stem along `angle` with a serif at each end.
fn caret(colors: &Palette, angle: f32) -> Drops {
    let color = colors["text"];
    let mut drops = vec![("body", capsule((-7.5, 0.0), (7.5, 0.0), 1.25, color))];
    drops.extend(ends(8.5, 3.0, 90.0, color));
    rotated(drops, angle)
}

pub const PALM: P = (0.0, 3.0);
const FINGERS: [&str; 4] = ["finger0", "finger1", "finger2", "finger3"];

/// Palm with a thumb and three fingers reaching about `reach` from its
/// center, tapering from `base` to `tip`.
fn hand(palm: f32, reach: f32, base: f32, tip: f32, color: Color) -> Drops {
    let mut drops = vec![("body", blob(PALM, palm, color))];
    let fingers = [(-165.0, 0.7), (-115.0, 1.0), (-90.0, 1.05), (-65.0, 0.95)];
    for (key, (angle, length)) in FINGERS.into_iter().zip(fingers) {
        let (start, end) = (polar(PALM.0, PALM.1, 3.5, angle), polar(PALM.0, PALM.1, reach * length, angle));
        drops.push((key, cone(start, base, end, tip, color).anchored(PALM)));
    }
    drops
}

pub const BADGE: P = (8.0, 7.5);

/// The fist with a badge drop at its lower right; `glyph` is carved into it.
fn badged(colors: &Palette, color: Color, glyph: &[(&'static str, P, P)]) -> Drops {
    let mut drops = hand(5.0, 6.4, 2.4, 2.6, colors["grab"]);
    drops.push(("badge", blob(BADGE, 4.0, color).own()));
    for &(key, (ax, ay), (bx, by)) in glyph {
        let ends = ((BADGE.0 + ax, BADGE.1 + ay), (BADGE.0 + bx, BADGE.1 + by));
        drops.push((key, capsule(ends.0, ends.1, 0.75, color).carve().anchored(BADGE)));
    }
    drops
}

/// Ring of `radius` around a carved hole, as in the ban sign.
fn ring(radius: f32, hole: f32, color: Color) -> Drops {
    vec![("body", blob((0.0, 0.0), radius, color)), ("hole", blob((0.0, 0.0), hole, color).carve())]
}

/// Center of the skull's jaw.
pub const JAW: P = (0.0, 5.4);

/// A skull with carved eyes, nose and teeth, its jaw a separate drop fused on
/// below.
fn skull(color: Color) -> Drops {
    let head = (0.0, -1.4);
    let mut drops = vec![
        ("body", blob(head, 7.2, color)),
        ("jaw", capsule((JAW.0 - 2.6, JAW.1), (JAW.0 + 2.6, JAW.1), 2.4, color).anchored(head)),
        ("eye_l", blob((-2.8, -1.0), 2.1, color).carve().own()),
        ("eye_r", blob((2.8, -1.0), 2.1, color).carve().own()),
        ("nose", cone((0.0, 1.7), 0.5, (0.0, 2.9), 0.9, color).carve().own()),
    ];
    for (key, x) in [("tooth_l", -1.3), ("tooth_r", 1.3)] {
        drops.push((key, capsule((x, JAW.1 - 1.2), (x, JAW.1 + 1.2), 0.4, color).carve().anchored(JAW)));
    }
    drops
}

/// Solid lens of `radius` with a handle and a carved bar, or a carved plus
/// when `plus`.
fn magnifier(colors: &Palette, radius: f32, plus: bool) -> Drops {
    let color = colors["zoom"];
    let start = polar(0.0, 0.0, 0.72 * radius, 45.0);
    let arm = 0.45 * radius;
    let mut drops = vec![
        ("body", blob((0.0, 0.0), radius, color)),
        ("handle", cone(start, 2.5, (start.0 + 4.9, start.1 + 4.9), 2.1, color).own()),
        ("plus_h", capsule((-arm, 0.0), (arm, 0.0), 1.1, color).carve()),
    ];
    if plus {
        drops.push(("plus_v", capsule((0.0, -arm), (0.0, arm), 1.1, color).carve()));
    }
    drops
}

/// A cell of `scale` at `center` that splits in two and merges back twice a
/// turn while turning half a turn, so the halves have swapped when it loops.
fn mitosis(colors: &Palette, phase: f32, center: P, scale: f32, keys: [&'static str; 2]) -> Drops {
    let split = (0.5 - 0.5 * (TAU * 2.0 * phase).cos()).powf(0.8);
    let radius = scale * (4.2 - 1.1 * split);
    keys.into_iter()
        .zip([(0.0, colors["busy"]), (180.0, colors["split"])])
        .map(|(key, (turn, color))| {
            let at = polar(center.0, center.1, scale * 5.2 * split, 180.0 * phase + turn);
            (key, blob(at, radius, color))
        })
        .collect()
}

/// The teardrop with a small dividing cell at its belly.
fn progress(colors: &Palette, phase: f32) -> Drops {
    let mut drops = vec![("body", teardrop(colors["arrow"]))];
    let cell = mitosis(colors, phase, (14.0, 15.0), 0.5, ["bud", "split"]);
    drops.extend(cell.into_iter().map(|(key, drop)| (key, drop.anchored(BELLY))));
    drops
}

/// Center of the help bud.
pub const BUD: P = (14.2, 15.2);

/// The teardrop with a bud at its belly, a question mark carved into it.
fn help(colors: &Palette) -> Drops {
    let (arrow, color) = (colors["arrow"], colors["help"]);
    let mut drops = vec![("body", teardrop(arrow)), ("bud", blob(BUD, 3.8, color).anchored(BELLY))];
    let at = |(x, y): P| (BUD.0 + x, BUD.1 + y);
    let mut hook: Vec<P> = (0..=5).map(|i| at(polar(0.0, -0.9, 1.2, 200.0 + 50.0 * i as f32))).collect();
    hook.push(at((0.0, 0.5)));
    let keys = ["query0", "query1", "query2", "query3", "query4", "query5"];
    for (key, ends) in keys.into_iter().zip(hook.windows(2)) {
        drops.push((key, capsule(ends[0], ends[1], 0.42, color).carve().anchored(BUD)));
    }
    drops.push(("query_dot", blob(at((0.0, 1.9)), 0.5, color).carve().anchored(BUD)));
    drops
}

fn wait(colors: &Palette, phase: f32) -> Drops {
    mitosis(colors, phase, (0.0, 0.0), 1.0, ["body", "split"])
}

/// Drops of a cursor; `phase` in [0, 1) drives busy loops.
pub fn layout(colors: &Palette, shape: Shape, phase: f32) -> Drops {
    match shape {
        Shape::Default => vec![("body", teardrop(colors["arrow"]))],
        Shape::Progress => progress(colors, phase),
        Shape::Wait => wait(colors, phase),
        Shape::Pointer => {
            let color = colors["link"];
            vec![
                ("body", cone((0.0, 0.9), 0.9, (0.0, 10.0), 5.8, color)),
                ("hole", blob((0.0, 10.0), 2.0, color).carve().own()),
            ]
        }
        Shape::Text => caret(colors, 90.0),
        Shape::VerticalText => caret(colors, 0.0),
        Shape::EwResize => resize(colors, 0.0),
        Shape::NsResize => resize(colors, 90.0),
        Shape::NeswResize => resize(colors, -45.0),
        Shape::NwseResize => resize(colors, 45.0),
        Shape::ColResize => {
            let color = colors["resize"];
            let mut drops = vec![("body", capsule((0.0, -8.5), (0.0, 8.5), 1.25, color))];
            drops.extend(ends(10.0, 3.8, 135.0, color));
            drops
        }
        Shape::Grab => hand(4.4, 11.5, 2.1, 1.7, colors["grab"]),
        Shape::Grabbing => hand(5.0, 6.4, 2.4, 2.6, colors["grab"]),
        Shape::Copy => badged(
            colors,
            colors["copy"],
            &[("badge_h", (-2.2, 0.0), (2.2, 0.0)), ("badge_v", (0.0, -2.2), (0.0, 2.2))],
        ),
        Shape::NoDrop => badged(colors, colors["refuse"], &[("badge_slash", (-1.8, -1.8), (1.8, 1.8))]),
        Shape::NotAllowed => {
            let color = colors["refuse"];
            let mut drops = ring(9.3, 6.0, color);
            drops.push(("slash", capsule((-4.4, -4.4), (4.4, 4.4), 1.4, color).inlay()));
            drops
        }
        Shape::Crosshair => {
            let color = colors["crosshair"];
            let mut drops = vec![("body", blob((0.0, 0.0), 1.8, color))];
            for (key, angle) in ["tick0", "tick1", "tick2", "tick3"].into_iter().zip([-90.0, 0.0, 90.0, 180.0]) {
                drops.push((key, capsule(polar(0.0, 0.0, 5.8, angle), polar(0.0, 0.0, 10.5, angle), 1.3, color)));
            }
            drops
        }
        Shape::ZoomIn => magnifier(colors, 8.2, true),
        Shape::ZoomOut => magnifier(colors, 6.6, false),
        Shape::Kill => skull(colors["refuse"]),
        Shape::Help => help(colors),
    }
}

pub fn scene(colors: &Palette, shape: Shape, phase: f32) -> Scene {
    Scene::new(layout(colors, shape, phase))
}

const BUSY_FRAMES: usize = 64;
const BUSY_DELAY: u32 = 30;

/// Frames of an ordinary cursor: a busy loop, or rest then an idle gesture.
pub fn frames(colors: &Palette, shape: Shape) -> Vec<(Scene, u32)> {
    if shape.is_busy() {
        return (0..BUSY_FRAMES).map(|i| (scene(colors, shape, i as f32 / BUSY_FRAMES as f32), BUSY_DELAY)).collect();
    }
    let (hold, count) = idle::timing(shape);
    let rest = scene(colors, shape, 0.0);
    std::iter::once((rest.clone(), hold))
        .chain((1..count).map(|i| (idle::gesture(shape, rest.clone(), i as f32 / count as f32), idle::STEP)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Fluid;
    use crate::scene::blend;
    use std::collections::HashMap;

    fn colors() -> Palette {
        Palette::load::<Fluid>("carburetor").unwrap()
    }

    fn fields(drop: Drop) -> Vec<f32> {
        let [r, g, b] = drop.color;
        vec![
            drop.a.0,
            drop.a.1,
            drop.ra,
            drop.b.0,
            drop.b.1,
            drop.rb,
            r,
            g,
            b,
            drop.anchor.0,
            drop.anchor.1,
            drop.fade,
            drop.rim,
        ]
    }

    #[test]
    fn keys_keep_one_layer_across_shapes() {
        let mut layers = HashMap::new();
        for shape in Shape::ALL {
            for (key, drop) in layout(&colors(), shape, 0.0) {
                assert_eq!(*layers.entry(key).or_insert(drop.layer), drop.layer, "{key} in {shape:?}");
            }
        }
    }

    #[test]
    fn reversed_transition_matches_forward_transition() {
        let colors = colors();
        for a in Shape::ALL {
            for b in Shape::ALL {
                let (first, last) = (scene(&colors, a, 0.0), scene(&colors, b, 0.0));
                for step in 0..=20 {
                    let t = step as f32 / 20.0;
                    let (forward, backward) = (blend(&first, &last, t), blend(&last, &first, 1.0 - t));
                    assert!((forward.goo - backward.goo).abs() < 1e-4);
                    assert_eq!(forward.drops.len(), backward.drops.len());
                    for ((key, x), (other, y)) in forward.drops.iter().zip(&backward.drops) {
                        assert_eq!(key, other);
                        for (p, q) in fields(*x).into_iter().zip(fields(*y)) {
                            assert!((p - q).abs() < 1e-3, "{a:?} -> {b:?} {key} at {t}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn blend_endpoints_match_shapes() {
        let colors = colors();
        let (a, b) = (scene(&colors, Shape::Grabbing, 0.0), scene(&colors, Shape::Copy, 0.0));
        let end = blend(&a, &b, 1.0);
        for (key, drop) in &b.drops {
            for (p, q) in fields(end.get(key).unwrap()).into_iter().zip(fields(*drop)) {
                assert!((p - q).abs() < 1e-4, "{key}");
            }
        }
    }
}
