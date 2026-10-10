//! Parametric cursor rig.
//!
//! Every cursor is a list of keyed parts in design units around the hotspot,
//! with y pointing down. Transitions interpolate the parameters
//! of parts sharing a key; unshared parts grow from or collapse into an anchor.

use cursor_core::Shape;
use cursor_core::motion::{ease, lerp, polar, window};

/// Progress windows for parts that exist on only one side. They mirror each
/// other, so playing A->B backwards is exactly the forward B->A motion.
const LEAVE: (f32, f32) = (0.0, 0.75);
const ENTER: (f32, f32) = (0.25, 1.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    Origin,
    Own,
    Point(f32, f32),
}

/// How an arc appears when its partner shape lacks it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Enter {
    /// Grows from its anchor.
    Expand,
    /// Contracts from a wider, vanishing ring.
    Ripple,
    /// Spirals out of the center at full width while sweeping open.
    Sweep,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// Round-capped bar centered on the part position. `length` includes the
    /// caps, so a bar no longer than its width is a disc. A `None` angle adopts
    /// the partner's angle.
    Bar { length: f32, angle: Option<f32> },
    /// Open arrowhead with its tip at the part position.
    Chevron { angle: f32, arm: f32 },
    /// Stroked arc centered on the part position; a 360 degree sweep is a ring.
    Arc { radius: f32, start: f32, sweep: f32, enter: Enter },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub kind: Kind,
    pub from: Anchor,
}

pub type Parts = Vec<(&'static str, Part)>;

fn bar(x: f32, y: f32, length: f32, width: f32, angle: Option<f32>) -> Part {
    Part { x, y, width, kind: Kind::Bar { length, angle }, from: Anchor::Origin }
}

fn dot(x: f32, y: f32, diameter: f32) -> Part {
    bar(x, y, diameter, diameter, None)
}

fn ring(radius: f32, width: f32) -> Part {
    let kind = Kind::Arc { radius, start: -90.0, sweep: 360.0, enter: Enter::Ripple };
    Part { x: 0.0, y: 0.0, width, kind, from: Anchor::Origin }
}

fn badge_ring() -> Part {
    let kind = Kind::Arc { radius: 3.0, start: -90.0, sweep: 360.0, enter: Enter::Expand };
    Part { x: 8.0, y: 8.0, width: 2.0, kind, from: Anchor::Own }
}

fn spinner(radius: f32, width: f32, phase: f32) -> Part {
    let kind = Kind::Arc { radius, start: -90.0 + 360.0 * phase, sweep: 270.0, enter: Enter::Sweep };
    Part { x: 0.0, y: 0.0, width, kind, from: Anchor::Origin }
}

fn own(part: Part) -> Part {
    Part { from: Anchor::Own, ..part }
}

fn arrows(body: Part, angles: [f32; 2], distance: f32, arm: f32) -> Parts {
    let mut parts = vec![("body", body)];
    for (key, angle) in ["arrow0", "arrow1"].into_iter().zip(angles) {
        let (x, y) = polar(0.0, 0.0, distance, angle);
        let kind = Kind::Chevron { angle, arm };
        parts.push((key, Part { x, y, width: 2.5, kind, from: Anchor::Origin }));
    }
    parts
}

fn hand(palm_y: f32, palm: f32, reach: f32, finger: f32) -> Parts {
    let mut parts = vec![("body", dot(0.0, palm_y, palm))];
    let keys = ["finger0", "finger1", "finger2", "finger3"];
    for (key, angle) in keys.into_iter().zip([-155.0, -115.0, -65.0, -25.0]) {
        let (x, y) = polar(0.0, palm_y, reach, angle);
        parts.push((key, Part { from: Anchor::Point(0.0, palm_y), ..dot(x, y, finger) }));
    }
    parts
}

fn fist() -> Parts {
    hand(2.0, 10.0, 5.0, 4.5)
}

fn magnifier() -> Parts {
    vec![
        ("lens", ring(7.0, 2.5)),
        ("handle", bar(7.5, 7.5, 8.5, 3.5, Some(45.0))),
        ("plus_h", own(bar(0.0, 0.0, 7.0, 2.5, Some(0.0)))),
    ]
}

/// Parts of this shape; `phase` in [0, 1) rotates spinners.
pub fn parts(shape: Shape, phase: f32) -> Parts {
    match shape {
        Shape::Default => vec![("body", dot(0.0, 0.0, 11.0))],
        Shape::Pointer => vec![("body", bar(0.0, 0.0, 15.0, 7.5, Some(55.0)))],
        Shape::Text => vec![("body", bar(0.0, 0.0, 20.0, 3.0, Some(90.0)))],
        Shape::VerticalText => vec![("body", bar(0.0, 0.0, 20.0, 3.0, Some(0.0)))],
        Shape::EwResize => arrows(dot(0.0, 0.0, 6.0), [0.0, 180.0], 10.0, 4.0),
        Shape::NsResize => arrows(dot(0.0, 0.0, 6.0), [-90.0, 90.0], 10.0, 4.0),
        Shape::NeswResize => arrows(dot(0.0, 0.0, 6.0), [-45.0, 135.0], 10.0, 4.0),
        Shape::NwseResize => arrows(dot(0.0, 0.0, 6.0), [-135.0, 45.0], 10.0, 4.0),
        Shape::ColResize => arrows(bar(0.0, 0.0, 18.0, 3.0, Some(90.0)), [0.0, 180.0], 10.5, 3.5),
        Shape::Grab => hand(2.5, 9.0, 8.5, 4.0),
        Shape::Grabbing => fist(),
        Shape::Copy => {
            let mut parts = fist();
            parts.push(("badge_h", own(bar(8.0, 8.0, 6.0, 2.5, Some(0.0)))));
            parts.push(("badge_v", own(bar(8.0, 8.0, 6.0, 2.5, Some(90.0)))));
            parts
        }
        Shape::NoDrop => {
            let mut parts = fist();
            parts.push(("badge_ring", badge_ring()));
            parts.push(("badge_slash", own(bar(8.0, 8.0, 5.5, 2.0, Some(45.0)))));
            parts
        }
        Shape::NotAllowed => {
            vec![("body", bar(0.0, 0.0, 17.0, 3.0, Some(45.0))), ("ring", ring(9.0, 3.0))]
        }
        Shape::Progress => vec![("body", dot(0.0, 0.0, 6.0)), ("spin", spinner(9.0, 2.5, phase))],
        Shape::Wait => vec![("spin", spinner(8.0, 4.0, phase))],
        Shape::Crosshair => {
            let mut parts = vec![("body", dot(0.0, 0.0, 3.0))];
            let keys = ["tick0", "tick1", "tick2", "tick3"];
            for (key, angle) in keys.into_iter().zip([-90.0, 0.0, 90.0, 180.0]) {
                let (x, y) = polar(0.0, 0.0, 8.0, angle);
                parts.push((key, bar(x, y, 6.0, 2.0, Some(angle))));
            }
            parts
        }
        Shape::ZoomIn => {
            let mut parts = magnifier();
            parts.push(("plus_v", own(bar(0.0, 0.0, 7.0, 2.5, Some(90.0)))));
            parts
        }
        Shape::ZoomOut => magnifier(),
    }
}

/// The collapsed state a part grows from or shrinks into.
pub fn hidden(part: Part) -> Part {
    let (x, y) = match part.from {
        Anchor::Origin => (0.0, 0.0),
        Anchor::Own => (part.x, part.y),
        Anchor::Point(x, y) => (x, y),
    };
    let kind = match part.kind {
        Kind::Bar { angle, .. } => Kind::Bar { length: 0.0, angle },
        Kind::Chevron { angle, .. } => Kind::Chevron { angle, arm: 0.0 },
        Kind::Arc { radius, start, sweep, enter } => match enter {
            Enter::Expand => Kind::Arc { radius: 0.0, start, sweep, enter },
            Enter::Ripple => {
                let kind = Kind::Arc { radius: radius * 1.5, start, sweep, enter };
                return Part { width: 0.0, kind, ..part };
            }
            Enter::Sweep => {
                return Part { kind: Kind::Arc { radius: 0.0, start, sweep: 0.0, enter }, ..part };
            }
        },
    };
    Part { x, y, width: 0.0, kind, ..part }
}

/// Interpolates two parts of the same kind.
pub fn mix(a: Part, b: Part, t: f32) -> Part {
    let kind = match (a.kind, b.kind) {
        (Kind::Bar { length: l0, angle: a0 }, Kind::Bar { length: l1, angle: a1 }) => Kind::Bar {
            length: lerp(l0, l1, t),
            angle: match (a0.or(a1), a1.or(a0)) {
                (Some(first), Some(last)) => Some(lerp(first, last, t)),
                _ => None,
            },
        },
        (Kind::Chevron { angle: a0, arm: r0 }, Kind::Chevron { angle: a1, arm: r1 }) => {
            Kind::Chevron { angle: lerp(a0, a1, t), arm: lerp(r0, r1, t) }
        }
        (Kind::Arc { radius: r0, start: s0, sweep: w0, enter }, Kind::Arc { radius: r1, start: s1, sweep: w1, .. }) => {
            Kind::Arc { radius: lerp(r0, r1, t), start: lerp(s0, s1, t), sweep: lerp(w0, w1, t), enter }
        }
        (first, last) => panic!("cannot mix {first:?} with {last:?}"),
    };
    Part { x: lerp(a.x, b.x, t), y: lerp(a.y, b.y, t), width: lerp(a.width, b.width, t), kind, from: a.from }
}

/// Parts at progress `t` in [0, 1] of the transition from `a` to `b`.
pub fn blend(a: &[(&'static str, Part)], b: &[(&'static str, Part)], t: f32) -> Parts {
    let find = |parts: &[(&'static str, Part)], key: &str| parts.iter().find(|(k, _)| *k == key).map(|(_, p)| *p);
    let keys = a.iter().map(|(k, _)| *k).chain(b.iter().map(|(k, _)| *k).filter(|k| find(a, k).is_none()));
    keys.map(|key| {
        let part = match (find(a, key), find(b, key)) {
            (Some(first), Some(last)) => mix(first, last, ease(t)),
            (Some(first), None) => mix(first, hidden(first), ease(window(t, LEAVE))),
            (None, Some(last)) => mix(hidden(last), last, ease(window(t, ENTER))),
            (None, None) => unreachable!(),
        };
        (key, part)
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn fields(part: Part) -> Vec<f32> {
        let kind = match part.kind {
            Kind::Bar { length, angle } => vec![length, angle.unwrap_or(0.0)],
            Kind::Chevron { angle, arm } => vec![angle, arm],
            Kind::Arc { radius, start, sweep, .. } => vec![radius, start, sweep],
        };
        [part.x, part.y, part.width].into_iter().chain(kind).collect()
    }

    #[test]
    fn keys_keep_one_kind_across_shapes() {
        let mut kinds = HashMap::new();
        for shape in Shape::ALL {
            for (key, part) in parts(shape, 0.0) {
                let kind = std::mem::discriminant(&part.kind);
                assert_eq!(*kinds.entry(key).or_insert(kind), kind, "{key} in {shape:?}");
            }
        }
    }

    #[test]
    fn reversed_transition_matches_forward_transition() {
        for a in Shape::ALL {
            for b in Shape::ALL {
                let (first, last) = (parts(a, 0.0), parts(b, 0.0));
                for step in 0..=20 {
                    let t = step as f32 / 20.0;
                    let forward: HashMap<_, _> = blend(&first, &last, t).into_iter().collect();
                    let backward: HashMap<_, _> = blend(&last, &first, 1.0 - t).into_iter().collect();
                    assert_eq!(forward.len(), backward.len());
                    for (key, part) in forward {
                        for (x, y) in fields(part).into_iter().zip(fields(backward[key])) {
                            assert!((x - y).abs() < 1e-4, "{a:?} -> {b:?} {key} at {t}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn blend_endpoints_match_shapes() {
        let (a, b) = (parts(Shape::Grab, 0.0), parts(Shape::Copy, 0.0));
        assert_eq!(blend(&a, &b, 0.0).iter().filter(|(_, p)| p.width > 0.0).count(), a.len());
        let end = blend(&a, &b, 1.0);
        for (key, part) in b {
            let (_, blended) = end.iter().find(|(k, _)| *k == key).unwrap();
            assert_eq!(fields(*blended), fields(part), "{key}");
        }
    }
}
