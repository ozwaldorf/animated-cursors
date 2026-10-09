//! Idle loops: each cursor rests, then plays a short liquid gesture that ends
//! where it started. Spins turn by a symmetry of the glyph, so loops close.

use std::f32::consts::PI;

use cursor_core::Shape;
use cursor_core::motion::{bump, ease, lerp, polar, spring, window, wobble};

use crate::cursors::{BADGE, BELLY, PALM, scene};
use crate::scene::{Drop, P, Scene, Warp, blob};

/// Gesture frame length in milliseconds.
pub const STEP: u32 = 30;

/// Milliseconds at rest, then gesture frames.
pub fn timing(shape: Shape) -> (u32, usize) {
    match shape {
        Shape::Default => (1600, 30),
        Shape::Text | Shape::VerticalText => (1800, 24),
        Shape::Pointer | Shape::Grab | Shape::Copy | Shape::NotAllowed => (1200, 24),
        _ => (1400, 20),
    }
}

fn each(mut scene: Scene, f: impl Fn(&str, Drop) -> Drop) -> Scene {
    scene.drops.iter_mut().for_each(|(key, drop)| *drop = f(key, *drop));
    scene
}

fn moved(drop: Drop, (dx, dy): P) -> Drop {
    drop.map(|(x, y)| (x + dx, y + dy))
}

fn turned(drop: Drop, pivot: P, degrees: f32) -> Drop {
    drop.map(|(x, y)| {
        let (dx, dy) = (x - pivot.0, y - pivot.1);
        polar(pivot.0, pivot.1, dx.hypot(dy), dy.atan2(dx).to_degrees() + degrees)
    })
}

/// Unit vector from `from` toward `to`.
fn toward(from: P, to: P) -> P {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx.hypot(dy).max(1e-4);
    (dx / length, dy / length)
}

/// Moves a drop `distance` further from `from`.
fn pushed(drop: Drop, from: P, distance: f32) -> Drop {
    let (x, y) = toward(from, drop.center());
    moved(drop, (x * distance, y * distance))
}

/// Squashes the scene by `amount` along `angle`, bulging across to keep volume.
fn squashed(scene: Scene, pivot: P, angle: f32, amount: f32) -> Scene {
    Scene { warp: Warp { pivot, angle, along: 1.0 - amount, across: 1.0 + 0.6 * amount }, ..scene }
}

/// Two quick pulses.
fn twice(u: f32) -> f32 {
    bump(u, 0.0, 0.45) + bump(u, 0.5, 0.45)
}

/// The idle gesture at progress `u` in (0, 1).
pub fn gesture(shape: Shape, u: f32) -> Scene {
    let rest = scene(shape, 0.0);
    let origin = (0.0, 0.0);
    match shape {
        // A drop sags from the belly and springs back with a jiggle.
        Shape::Default => {
            let body = rest.get("body").expect("teardrop");
            let sag = bump(window(u, (0.0, 0.6)), 0.0, 1.0);
            let drip = blob((BELLY.0, BELLY.1 + 3.5 + 4.5 * sag), 1.9 * sag.sqrt(), body.color);
            let mut scene = rest;
            scene.drops.push(("drip", drip));
            squashed(scene, BELLY, 90.0, 0.06 * wobble(window(u, (0.45, 1.0)), 2.0))
        }
        // Two taps, flattening against the tip.
        Shape::Pointer => squashed(rest, origin, 90.0, 0.1 * twice(u)),
        // A bead of liquid runs down the stem.
        Shape::Text | Shape::VerticalText => {
            let stem = rest.get("body").expect("stem");
            let along = ease(u);
            let at = (lerp(stem.a.0, stem.b.0, along), lerp(stem.a.1, stem.b.1, along));
            let mut scene = rest;
            scene.drops.push(("bead", blob(at, 2.0 * (PI * u).sin(), stem.color)));
            scene
        }
        // Arrowheads pull off the ends of the bar and snap back.
        Shape::EwResize | Shape::NsResize | Shape::NeswResize | Shape::NwseResize | Shape::ColResize => {
            let pull = 0.9 * twice(u);
            each(rest, |key, d| match key.starts_with("arrow") {
                true => {
                    let (x, y) = toward(origin, d.a);
                    moved(d, (x * pull, y * pull))
                }
                false => d,
            })
        }
        // Fingers stretch out one after another.
        Shape::Grab => each(rest, |key, d| match key.strip_prefix("finger") {
            Some(i) => {
                let reach = 1.8 * bump(u, 0.12 * i.parse::<f32>().unwrap_or(0.0), 0.45);
                let (x, y) = toward(d.a, d.b);
                Drop { b: (d.b.0 + x * reach, d.b.1 + y * reach), ..d }
            }
            None => d,
        }),
        Shape::Grabbing => {
            let k = 1.0 - 0.08 * twice(u);
            Scene { warp: Warp { pivot: PALM, angle: 0.0, along: k, across: k }, ..rest }
        }
        // The badge drop tugs away from the fist while its plus turns.
        Shape::Copy => {
            let (x, y) = toward(PALM, BADGE);
            let tug = 1.2 * bump(u, 0.0, 0.6);
            let turn = 90.0 * spring(window(u, (0.3, 1.0)));
            each(rest, |key, d| match key {
                "badge" => moved(d, (x * tug, y * tug)),
                _ if key.starts_with("badge") => moved(turned(d, d.anchor, turn), (x * tug, y * tug)),
                _ => d,
            })
        }
        Shape::NoDrop => {
            let shake = 0.8 * (PI * 6.0 * u).sin() * (1.0 - u);
            each(rest, |key, d| if key.starts_with("badge") { moved(d, (shake, 0.0)) } else { d })
        }
        // The whole sign wobbles like jelly.
        Shape::NotAllowed => squashed(rest, origin, 0.0, 0.07 * wobble(u, 2.0)),
        // Ticks fall into the center, merge, and pull back out.
        Shape::Crosshair => {
            let pull = -3.4 * bump(u, 0.0, 1.0);
            each(rest, |key, d| if key.starts_with("tick") { pushed(d, origin, pull) } else { d })
        }
        Shape::ZoomIn => {
            each(rest, |key, d| if key.starts_with("plus") { turned(d, origin, 90.0 * spring(u)) } else { d })
        }
        Shape::ZoomOut => each(rest, |key, d| if key == "plus_h" { turned(d, origin, 180.0 * spring(u)) } else { d }),
        Shape::Progress | Shape::Wait => rest,
    }
}
