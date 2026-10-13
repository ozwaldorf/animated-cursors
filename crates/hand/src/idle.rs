//! Idle loops: each cursor rests, then plays a short gesture that ends where
//! it started. Spins turn by a symmetry of the prop, so loops close.

use std::f32::consts::{PI, TAU};

use cursor_core::Shape;
use cursor_core::motion::{bump, ease, window, wobble};

use crate::rig::{GRIP, Pose, Prop, Scene, WRIST, index_tip, palm_facing, reach, reach_root};
use crate::vector::{Rot, V};

/// Gesture frame length in milliseconds.
pub const STEP: u32 = 30;

/// Milliseconds at rest, then gesture frames.
pub fn timing(shape: Shape) -> (u32, usize) {
    match shape {
        Shape::Default => (1600, 30),
        Shape::Text | Shape::VerticalText => (1800, 28),
        Shape::Pointer | Shape::Grab | Shape::Copy => (1200, 28),
        Shape::NotAllowed => (0, 54),
        _ => (1400, 24),
    }
}

fn props(mut scene: Scene, f: impl Fn(Prop) -> Prop) -> Scene {
    scene.props.iter_mut().for_each(|(_, p)| *p = f(*p));
    scene
}

fn hand(scene: Scene, f: impl Fn(Pose) -> Pose) -> Scene {
    Scene { hand: f(scene.hand), ..scene }
}

/// The whole scene rolled `degrees` about the hand's grip, so what it holds
/// swings in the hand.
fn rolled(scene: Scene, degrees: f32) -> Scene {
    let (turn, pivot) = (Rot::z(degrees), scene.hand.place(GRIP));
    let about = |at: V| pivot + turn.apply(at - pivot);
    let scene = hand(scene, |p| Pose { at: about(p.at), roll: p.roll + degrees, ..p });
    props(scene, |p| Prop { at: about(p.at), roll: p.roll + degrees, ..p })
}

/// Curl of each finger changed by `f(finger)`.
fn curled(pose: Pose, f: impl Fn(usize) -> f32) -> Pose {
    Pose { curl: std::array::from_fn(|i| pose.curl[i] + f(i)), ..pose }
}

/// The whole scene moved by `offset`.
fn moved(scene: Scene, offset: V) -> Scene {
    let scene = hand(scene, |p| Pose { at: p.at + offset, ..p });
    props(scene, |p| Prop { at: p.at + offset, ..p })
}

/// `pose` moved so its index fingertip stays where it is in `rest`.
fn kept(pose: Pose, rest: &Pose) -> Pose {
    pose.pinned(index_tip(&pose), rest.place(index_tip(rest)))
}

/// Damped shake that starts and ends at rest.
fn shake(u: f32, cycles: f32) -> f32 {
    (PI * 2.0 * cycles * u).sin() * (1.0 - u)
}

/// Segments of the idle loop in play order: `Some(variant)` rests then plays
/// that gesture, `None` only rests for one hover bob. Each distinct segment
/// is stored once, so a long, shuffled pattern costs no more than its parts.
pub fn pattern(shape: Shape) -> Vec<Option<usize>> {
    match shape {
        Shape::Default => vec![
            Some(0),
            None,
            Some(2),
            Some(1),
            None,
            None,
            Some(0),
            Some(2),
            None,
            Some(1),
            Some(0),
            None,
            Some(2),
            None,
            Some(1),
        ],
        Shape::Pointer => vec![Some(1), Some(0), Some(1), Some(1), Some(0)],
        Shape::Grab => vec![Some(0), Some(2), Some(1), Some(0), Some(1), Some(2), Some(1)],
        _ => vec![Some(0)],
    }
}

/// The idle gesture `variant` of `rest`, the cursor's resting scene, at
/// progress `u` in (0, 1).
pub fn gesture(shape: Shape, rest: Scene, u: f32, variant: usize) -> Scene {
    match shape {
        // The hand itself stays where it floats while, by turns, the fingers
        // curl inward with the thumb closing in toward them, the hand splays
        // open a little, and a ripple runs from the pinky to the index with the
        // thumb last.
        Shape::Default => hand(rest, |p| {
            let thumb = |out: f32, fold: f32, bend: f32| [p.thumb[0] + out, p.thumb[1] + fold, p.thumb[2] + bend];
            match variant {
                0 => {
                    let close = bump(u, 0.0, 1.0);
                    let spread = p.spread.map(|s| s * (1.0 - 0.2 * close));
                    Pose {
                        spread,
                        thumb: thumb(-18.0 * close, 6.0 * close, 0.45 * close),
                        ..curled(p, |_| 0.2 * close)
                    }
                }
                1 => {
                    let splay = bump(u, 0.0, 1.0);
                    let spread = p.spread.map(|s| s * (1.0 + 0.4 * splay));
                    Pose {
                        spread,
                        thumb: thumb(12.0 * splay, -6.0 * splay, -0.1 * splay),
                        ..curled(p, |_| -0.12 * splay)
                    }
                }
                _ => {
                    let ripple = curled(p, |i| 0.18 * bump(u, 0.12 * (3 - i) as f32, 0.45));
                    Pose { thumb: thumb(0.0, 0.0, 0.35 * bump(u, 0.5, 0.45)), ..ripple }
                }
            }
        }),
        // A tap: the index straightens to lift its tip off the hotspot, then
        // strikes back down and bounces once.
        Shape::Pointer if variant == 1 => hand(rest, |p| {
            let lift = ease(window(u, (0.0, 0.5))) - ease(window(u, (0.55, 0.68))) + 0.15 * bump(u, 0.68, 0.2);
            let mut flex = p.flex;
            flex[0] = p.flex[0].map(|bend| bend * (1.0 - 0.9 * lift));
            Pose { flex, ..p }
        }),
        // Two quick pokes: the hand thrusts along the finger into the screen
        // while the fingertip stays planted on the hotspot, so the finger
        // flexes under the press.
        Shape::Pointer => hand(rest, |p| {
            let thrust = bump(u, 0.05, 0.3) + bump(u, 0.5, 0.3);
            let tip = p.place(index_tip(&p));
            let along = (tip - p.place(reach_root(0))).normalized();
            let pushed = Pose { at: p.at + along * (1.2 * thrust), size: p.size * (1.0 - 0.04 * thrust), ..p };
            let mut flex = pushed.flex;
            flex[0] = reach(&pushed, 0, tip);
            Pose { flex, ..pushed }
        }),
        // The flat hand chops twice: each time it snaps toward the viewer from
        // the wrist and down, and after the second it pulls back with a little
        // bounce.
        Shape::Text | Shape::VerticalText => hand(rest, |p| {
            let chop =
                |start: f32| ease(window(u, (start, start + 0.08))) - ease(window(u, (start + 0.08, start + 0.22)));
            let strike = chop(0.05) + chop(0.3);
            let back = window(u, (0.52, 0.95));
            let bounce = 0.3 * (TAU * 2.0 * back).sin() * (1.0 - back);
            let chopped = Pose { tip: p.tip - 40.0 * strike, ..p }.pinned(WRIST, p.place(WRIST));
            Pose { at: chopped.at + V(0.0, strike + bounce, 0.0), ..chopped }
        }),
        // The pinch tugs a short way across its beak and back, as if dragging
        // an edge.
        Shape::EwResize | Shape::NsResize | Shape::NeswResize | Shape::NwseResize | Shape::ColResize => {
            hand(rest, |p| {
                let beak = palm_facing(&p);
                let drag = 1.2 * (TAU * u).sin() * bump(u, 0.0, 1.0).sqrt();
                Pose { at: p.at + V(-beak.1, beak.0, 0.0).normalized() * drag, ..p }
            })
        }
        // By turns, fingers wave one after another as the hand rocks, all four
        // beckon twice together, and the hand stretches wide open.
        Shape::Grab => hand(rest, |p| match variant {
            0 => {
                let wave = curled(p, |i| 0.15 * bump(u, 0.1 * i as f32, 0.45));
                Pose { turn: p.turn + 10.0 * wobble(u, 1.5), ..wave }
            }
            1 => {
                let beckon = bump(u, 0.05, 0.4) + bump(u, 0.5, 0.4);
                Pose { thumb: [p.thumb[0] - 10.0 * beckon, p.thumb[1], p.thumb[2]], ..curled(p, |_| 0.45 * beckon) }
            }
            _ => {
                let stretch = bump(u, 0.0, 1.0);
                Pose {
                    spread: p.spread.map(|s| s * (1.0 + 0.7 * stretch)),
                    thumb: [p.thumb[0] + 20.0 * stretch, p.thumb[1], p.thumb[2]],
                    ..curled(p, |_| -0.1 * stretch)
                }
            }
        }),
        Shape::Grabbing => hand(rest, |p| Pose { roll: p.roll + 8.0 * shake(u, 3.0), ..p }),
        // The badge flips over as the fist bobs.
        Shape::Copy => {
            let scene = props(rest, |p| Prop { spin: 360.0 * ease(u), ..p });
            hand(scene, |p| Pose { tip: p.tip - 20.0 * bump(u, 0.0, 1.0), ..p })
        }
        Shape::NoDrop => moved(rest, V(1.2 * shake(u, 3.0), 0.0, 0.0)),
        // The raised finger wags steadily from its knuckle, lingering at each
        // side and snapping through the middle, bending a little as it whips.
        // The hand tilts a little after it.
        Shape::NotAllowed => hand(rest, |p| {
            let angle = TAU * 3.0 * u;
            let swing = angle.sin().signum() * angle.sin().abs().powf(0.6);
            let mut spread = p.spread;
            spread[0] += swing * if swing > 0.0 { 22.0 } else { 16.0 };
            let mut flex = p.flex;
            let whip = [0.0, 4.0, 2.8].map(|bend| bend * angle.cos().powi(2));
            flex[0] = std::array::from_fn(|j| p.flex[0][j] + whip[j]);
            let knuckle = p.place(reach_root(0));
            Pose { spread, flex, roll: p.roll + 3.0 * (angle - 0.6).sin(), ..p }.pinned(reach_root(0), knuckle)
        }),
        // The hand sweeps the reticle around, as if taking aim.
        Shape::Crosshair => rolled(rest, 14.0 * wobble(u, 1.5)),
        Shape::ZoomIn | Shape::ZoomOut => rolled(rest, 12.0 * wobble(u, 2.0)),
        Shape::Progress | Shape::Wait => rest,
    }
}

/// Milliseconds per hover bob, roughly; each segment fits whole bobs.
pub const BOB: u32 = 2200;

/// The scene floating at `phase` in [0, 1) of an idle segment `length`
/// milliseconds long: the pointer sways on its fingertip, default drifts
/// around the hotspot, and the rest bob up and down with what they hold.
pub fn hover(shape: Shape, scene: Scene, phase: f32, length: u32) -> Scene {
    let cycles = (length as f32 / BOB as f32).round().max(1.0);
    let (s, c) = (2.0 * PI * cycles * phase).sin_cos();
    match shape {
        Shape::Pointer => {
            let still = scene.hand;
            hand(scene, |p| kept(Pose { turn: p.turn + 7.0 * s, roll: p.roll + 2.5 * (1.0 - c), ..p }, &still))
        }
        // The resting hand drifts a little around the hotspot.
        Shape::Default => {
            let scene = moved(scene, V(0.3 * s * c, -0.35 * s, 0.0));
            hand(scene, |p| Pose { turn: p.turn + 6.0 * s, roll: p.roll + 1.5 * s, ..p })
        }
        _ => {
            let scene = moved(scene, V(0.0, -0.5 * s, 0.0));
            hand(scene, |p| Pose { tip: p.tip + 5.0 * s, roll: p.roll + 1.5 * s, ..p })
        }
    }
}
