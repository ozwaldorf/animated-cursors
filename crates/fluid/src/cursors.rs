//! Each cursor as a scene of liquid drops, in design units with the hotspot
//! at the origin.

use std::f32::consts::TAU;

use cursor_core::Shape;
use cursor_core::color::{Color, hex};
use cursor_core::motion::polar;

use crate::idle;
use crate::scene::{Drop, Drops, P, Scene, blob, capsule, cone};

/// Carburetor palette: one blue for every cursor, with its lighter blues for
/// busy drops and status hues only where they carry meaning.
pub const BLUE: u32 = 0x4589ff;
pub const SAPPHIRE: u32 = 0x78a9ff;
pub const SKY: u32 = 0x82cffe;
pub const GREEN: u32 = 0x42be65;
pub const RED: u32 = 0xfa4d56;

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

/// Chevrons pointing along 0 and 180 degrees, tips `reach` from the origin.
/// Each arm starts at its tip.
fn chevrons(reach: f32, arm: f32, color: Color) -> Drops {
    let mut drops = Drops::new();
    for (keys, angle) in ARROWS.into_iter().zip([0.0, 180.0]) {
        let tip = polar(0.0, 0.0, reach, angle);
        for (key, turn) in keys.into_iter().zip([135.0, -135.0]) {
            drops.push((key, capsule(tip, polar(tip.0, tip.1, arm, angle + turn), 1.15, color)));
        }
    }
    drops
}

/// Bar along `angle` with an arrowhead at each end.
fn resize(angle: f32) -> Drops {
    let color = hex(BLUE);
    let mut drops = vec![("body", capsule((-8.5, 0.0), (8.5, 0.0), 1.15, color))];
    drops.extend(chevrons(9.5, 4.4, color));
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
fn badged(color: u32, glyph: &[(&'static str, P, P)]) -> Drops {
    let mut drops = hand(5.0, 6.4, 2.4, 2.6, hex(BLUE));
    drops.push(("badge", blob(BADGE, 4.0, hex(color)).own()));
    for &(key, (ax, ay), (bx, by)) in glyph {
        let ends = ((BADGE.0 + ax, BADGE.1 + ay), (BADGE.0 + bx, BADGE.1 + by));
        drops.push((key, capsule(ends.0, ends.1, 0.75, hex(color)).carve().anchored(BADGE)));
    }
    drops
}

/// Ring of `radius` around a carved hole, as in the zoom lens and the ban sign.
fn ring(radius: f32, hole: f32, color: Color) -> Drops {
    vec![("body", blob((0.0, 0.0), radius, color)), ("hole", blob((0.0, 0.0), hole, color).carve())]
}

fn magnifier() -> Drops {
    let color = hex(BLUE);
    let mut drops = ring(7.8, 5.0, color);
    drops.push(("handle", cone((5.6, 5.6), 2.5, (10.5, 10.5), 2.1, color).own()));
    drops.push(("plus_h", capsule((-2.4, 0.0), (2.4, 0.0), 1.0, color).inlay()));
    drops
}

const ORBIT: [&str; 3] = ["orbit0", "orbit1", "orbit2"];

/// Two drops circling the teardrop's belly, dipping in and out of it.
fn progress(phase: f32) -> Drops {
    let center = (12.0, 14.5);
    let mut drops = vec![("body", teardrop(hex(BLUE)))];
    for (i, (key, color)) in ORBIT.into_iter().zip([SAPPHIRE, SKY]).enumerate() {
        let at = polar(center.0, center.1, 3.0, 360.0 * phase + 180.0 * i as f32 - 135.0);
        drops.push((key, blob(at, 1.8, hex(color)).anchored(BELLY)));
    }
    drops
}

/// Three drops orbiting a core, each falling in and flung out twice a turn.
fn wait(phase: f32) -> Drops {
    let mut drops = vec![("body", blob((0.0, 0.0), 3.0, hex(BLUE)))];
    for (i, (key, color)) in ORBIT.into_iter().zip([SAPPHIRE, SKY, SAPPHIRE]).enumerate() {
        let offset = i as f32 / 3.0;
        let reach = 6.5 + 2.5 * (TAU * (2.0 * phase + offset)).cos();
        let at = polar(0.0, 0.0, reach, 360.0 * (phase + offset) - 90.0);
        drops.push((key, blob(at, 2.5, hex(color))));
    }
    drops
}

/// Drops of a cursor; `phase` in [0, 1) drives busy loops.
pub fn layout(shape: Shape, phase: f32) -> Drops {
    match shape {
        Shape::Default => vec![("body", teardrop(hex(BLUE)))],
        Shape::Progress => progress(phase),
        Shape::Wait => wait(phase),
        Shape::Pointer => {
            let color = hex(BLUE);
            vec![
                ("body", cone((0.0, 0.9), 0.9, (0.0, 10.0), 5.8, color)),
                ("hole", blob((0.0, 10.0), 2.0, color).carve().own()),
            ]
        }
        Shape::Text | Shape::VerticalText => {
            let color = hex(BLUE);
            let drops = vec![
                ("body", capsule((0.0, -7.5), (0.0, 7.5), 1.25, color)),
                ("cap0", capsule((-3.0, -8.5), (3.0, -8.5), 1.2, color).own()),
                ("cap1", capsule((-3.0, 8.5), (3.0, 8.5), 1.2, color).own()),
            ];
            if shape == Shape::Text { drops } else { rotated(drops, 90.0) }
        }
        Shape::EwResize => resize(0.0),
        Shape::NsResize => resize(90.0),
        Shape::NeswResize => resize(-45.0),
        Shape::NwseResize => resize(45.0),
        Shape::ColResize => {
            let color = hex(BLUE);
            let mut drops = vec![("body", capsule((0.0, -8.5), (0.0, 8.5), 1.25, color))];
            drops.extend(chevrons(10.0, 3.8, color));
            drops
        }
        Shape::Grab => hand(4.4, 11.5, 2.1, 1.7, hex(BLUE)),
        Shape::Grabbing => hand(5.0, 6.4, 2.4, 2.6, hex(BLUE)),
        Shape::Copy => badged(GREEN, &[("badge_h", (-2.2, 0.0), (2.2, 0.0)), ("badge_v", (0.0, -2.2), (0.0, 2.2))]),
        Shape::NoDrop => badged(RED, &[("badge_slash", (-1.8, -1.8), (1.8, 1.8))]),
        Shape::NotAllowed => {
            let color = hex(RED);
            let mut drops = ring(9.3, 6.0, color);
            drops.push(("slash", capsule((-4.4, -4.4), (4.4, 4.4), 1.4, color).inlay()));
            drops
        }
        Shape::Crosshair => {
            let color = hex(BLUE);
            let mut drops = vec![("body", blob((0.0, 0.0), 1.8, color))];
            for (key, angle) in ["tick0", "tick1", "tick2", "tick3"].into_iter().zip([-90.0, 0.0, 90.0, 180.0]) {
                drops.push((key, capsule(polar(0.0, 0.0, 5.8, angle), polar(0.0, 0.0, 10.5, angle), 1.3, color)));
            }
            drops
        }
        Shape::ZoomIn => {
            let mut drops = magnifier();
            drops.push(("plus_v", capsule((0.0, -2.4), (0.0, 2.4), 1.0, hex(BLUE)).inlay()));
            drops
        }
        Shape::ZoomOut => magnifier(),
    }
}

pub fn scene(shape: Shape, phase: f32) -> Scene {
    Scene::new(layout(shape, phase))
}

const BUSY_FRAMES: usize = 40;
const BUSY_DELAY: u32 = 30;

/// Frames of an ordinary cursor: a busy loop, or rest then an idle gesture.
pub fn frames(shape: Shape) -> Vec<(Scene, u32)> {
    if shape.is_busy() {
        return (0..BUSY_FRAMES).map(|i| (scene(shape, i as f32 / BUSY_FRAMES as f32), BUSY_DELAY)).collect();
    }
    let (hold, count) = idle::timing(shape);
    std::iter::once((scene(shape, 0.0), hold))
        .chain((1..count).map(|i| (idle::gesture(shape, i as f32 / count as f32), idle::STEP)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::blend;
    use std::collections::HashMap;

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
            for (key, drop) in layout(shape, 0.0) {
                assert_eq!(*layers.entry(key).or_insert(drop.layer), drop.layer, "{key} in {shape:?}");
            }
        }
    }

    #[test]
    fn reversed_transition_matches_forward_transition() {
        for a in Shape::ALL {
            for b in Shape::ALL {
                let (first, last) = (scene(a, 0.0), scene(b, 0.0));
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
        let (a, b) = (scene(Shape::Grabbing, 0.0), scene(Shape::Copy, 0.0));
        let end = blend(&a, &b, 1.0);
        for (key, drop) in &b.drops {
            for (p, q) in fields(end.get(key).unwrap()).into_iter().zip(fields(*drop)) {
                assert!((p - q).abs() < 1e-4, "{key}");
            }
        }
    }
}
