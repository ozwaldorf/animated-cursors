//! Idle loops: each cursor rests, then plays a short gesture that ends where
//! it started. Spins turn by a symmetry of the part, so loops close cleanly.

use cursor_core::Shape;
use cursor_core::motion::{bump, polar, spring, wobble};

use crate::rig::{Kind, Part, Parts, parts};

/// Gesture frame length in milliseconds.
pub const STEP: u32 = 30;

/// Milliseconds at rest, then gesture frames.
pub fn timing(shape: Shape) -> (u32, usize) {
    match shape {
        Shape::Default => (1600, 22),
        Shape::Text | Shape::VerticalText => (1800, 18),
        Shape::Pointer | Shape::Grab | Shape::Copy | Shape::NotAllowed => (1200, 24),
        _ => (1400, 20),
    }
}

/// Scales a part about `pivot`.
fn scaled(part: Part, pivot: (f32, f32), k: f32) -> Part {
    let kind = match part.kind {
        Kind::Bar { length, angle } => Kind::Bar { length: length * k, angle },
        Kind::Chevron { angle, arm } => Kind::Chevron { angle, arm: arm * k },
        Kind::Arc { radius, start, sweep, enter } => Kind::Arc { radius: radius * k, start, sweep, enter },
    };
    let (x, y) = (pivot.0 + (part.x - pivot.0) * k, pivot.1 + (part.y - pivot.1) * k);
    Part { x, y, width: part.width * k, kind, ..part }
}

/// Turns a part by `degrees` about `pivot`.
fn turned(part: Part, pivot: (f32, f32), degrees: f32) -> Part {
    let (dx, dy) = (part.x - pivot.0, part.y - pivot.1);
    let (x, y) = polar(pivot.0, pivot.1, dx.hypot(dy), dy.atan2(dx).to_degrees() + degrees);
    let kind = match part.kind {
        Kind::Bar { length, angle } => Kind::Bar { length, angle: Some(angle.unwrap_or(0.0) + degrees) },
        Kind::Chevron { angle, arm } => Kind::Chevron { angle: angle + degrees, arm },
        Kind::Arc { radius, start, sweep, enter } => Kind::Arc { radius, start: start + degrees, sweep, enter },
    };
    Part { x, y, kind, ..part }
}

/// Moves a part `distance` further from `from`.
fn pushed(part: Part, from: (f32, f32), distance: f32) -> Part {
    let (dx, dy) = (part.x - from.0, part.y - from.1);
    let length = dx.hypot(dy).max(1e-4);
    Part { x: part.x + dx / length * distance, y: part.y + dy / length * distance, ..part }
}

fn each(rest: Parts, f: impl Fn(&str, Part) -> Part) -> Parts {
    rest.into_iter().map(|(key, part)| (key, f(key, part))).collect()
}

/// The idle gesture at progress `u` in (0, 1).
pub fn gesture(shape: Shape, u: f32) -> Parts {
    let rest = parts(shape, 0.0);
    let origin = (0.0, 0.0);
    match shape {
        // Two heartbeats.
        Shape::Default => {
            each(rest, |_, p| scaled(p, origin, 1.0 + 0.12 * bump(u, 0.0, 0.4) + 0.07 * bump(u, 0.4, 0.4)))
        }
        // The ring ripples out while the dot pulses.
        Shape::Pointer => each(rest, |key, p| match (key, p.kind) {
            ("ring", Kind::Arc { radius, start, sweep, enter }) => {
                let swell = bump(u, 0.0, 0.8);
                let kind = Kind::Arc { radius: radius + 1.3 * swell, start, sweep, enter };
                Part { width: p.width * (1.0 - 0.3 * swell), kind, ..p }
            }
            _ => scaled(p, origin, 1.0 + 0.18 * bump(u, 0.1, 0.5)),
        }),
        Shape::Text | Shape::VerticalText => each(rest, |_, p| match p.kind {
            Kind::Bar { length, angle } => {
                Part { kind: Kind::Bar { length: length * (1.0 + 0.1 * wobble(u, 1.0)), angle }, ..p }
            }
            _ => p,
        }),
        Shape::EwResize | Shape::NsResize | Shape::NeswResize | Shape::NwseResize | Shape::ColResize => {
            let nudge = 0.8 * (bump(u, 0.0, 0.45) + bump(u, 0.5, 0.45));
            each(rest, |key, p| if key.starts_with("arrow") { pushed(p, origin, nudge) } else { p })
        }
        // Fingers lift off the palm one after another.
        Shape::Grab => each(rest, |key, p| match key.strip_prefix("finger") {
            Some(i) => pushed(p, (0.0, 2.5), 1.3 * bump(u, 0.12 * i.parse::<f32>().unwrap_or(0.0), 0.45)),
            None => p,
        }),
        Shape::Grabbing => {
            let squeeze = 1.0 - 0.09 * (bump(u, 0.0, 0.45) + bump(u, 0.5, 0.45));
            each(rest, |_, p| scaled(p, (0.0, 2.0), squeeze))
        }
        Shape::Copy => {
            each(rest, |key, p| if key.starts_with("badge") { turned(p, (8.0, 8.0), 90.0 * spring(u)) } else { p })
        }
        Shape::NoDrop => {
            let shake = 0.6 * (std::f32::consts::TAU * 3.0 * u).sin() * (1.0 - u);
            each(rest, |key, p| if key.starts_with("badge") { Part { x: p.x + shake, ..p } } else { p })
        }
        // The slash flips end over end; a bar looks the same half a turn on.
        Shape::NotAllowed => each(rest, |key, p| if key == "body" { turned(p, origin, 180.0 * spring(u)) } else { p }),
        Shape::Crosshair => {
            let pull = -1.4 * bump(u, 0.0, 1.0);
            each(rest, |key, p| if key.starts_with("tick") { pushed(p, origin, pull) } else { p })
        }
        Shape::ZoomIn => {
            each(rest, |key, p| if key.starts_with("plus") { turned(p, origin, 90.0 * spring(u)) } else { p })
        }
        Shape::ZoomOut => each(rest, |key, p| if key == "plus_h" { turned(p, origin, 180.0 * spring(u)) } else { p }),
        Shape::Progress | Shape::Wait => rest,
    }
}
