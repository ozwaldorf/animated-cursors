//! Scenes of keyed liquid drops, and the reversible blend between them.
//!
//! Drops are in design units with the hotspot at the origin. Drops sharing a
//! key flow into each other; the rest bud out of or melt into their anchors.

use std::f32::consts::PI;

use cursor_core::color::{Color, mix as mix_color};
use cursor_core::motion::{ease, lerp, window};

pub type P = (f32, f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// Melts together with every other body drop.
    Body,
    /// Cut out of the bodies, leaving a white glyph.
    Carve,
    /// Liquid set inside carved holes.
    Inlay,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drop {
    /// Round cone from a circle of radius `ra` at `a` to one of radius `rb`
    /// at `b`; equal ends make a circle, equal radii a capsule.
    pub a: P,
    pub ra: f32,
    pub b: P,
    pub rb: f32,
    pub layer: Layer,
    pub color: Color,
    /// Where the drop melts away when the other scene lacks it.
    pub anchor: P,
    /// Added to the drop's distance; large values dissolve it.
    pub fade: f32,
    /// Fraction of the white border drawn around this drop.
    pub rim: f32,
}

pub fn cone(a: P, ra: f32, b: P, rb: f32, color: Color) -> Drop {
    Drop { a, ra, b, rb, layer: Layer::Body, color, anchor: (0.0, 0.0), fade: 0.0, rim: 1.0 }
}

pub fn capsule(a: P, b: P, r: f32, color: Color) -> Drop {
    cone(a, r, b, r, color)
}

pub fn blob(center: P, r: f32, color: Color) -> Drop {
    cone(center, r, center, r, color)
}

impl Drop {
    pub fn carve(self) -> Drop {
        Drop { layer: Layer::Carve, ..self }
    }

    pub fn inlay(self) -> Drop {
        Drop { layer: Layer::Inlay, ..self }
    }

    pub fn anchored(self, anchor: P) -> Drop {
        Drop { anchor, ..self }
    }

    /// Melts away in place.
    pub fn own(self) -> Drop {
        self.anchored(self.center())
    }

    pub fn center(&self) -> P {
        ((self.a.0 + self.b.0) / 2.0, (self.a.1 + self.b.1) / 2.0)
    }

    /// Moves both ends; radii and anchor stay.
    pub fn map(self, f: impl Fn(P) -> P) -> Drop {
        Drop { a: f(self.a), b: f(self.b), ..self }
    }
}

/// Squashes the whole scene about `pivot`, scaling by `along` in the
/// direction `angle` (degrees) and by `across` perpendicular to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Warp {
    pub pivot: P,
    pub angle: f32,
    pub along: f32,
    pub across: f32,
}

impl Warp {
    pub const NONE: Warp = Warp { pivot: (0.0, 0.0), angle: 0.0, along: 1.0, across: 1.0 };

    /// Maps a point through the warp, or back through it when `inverse`.
    pub fn apply(&self, (x, y): P, inverse: bool) -> P {
        let (s, c) = self.angle.to_radians().sin_cos();
        let (dx, dy) = (x - self.pivot.0, y - self.pivot.1);
        let (u, v) = (dx * c + dy * s, dy * c - dx * s);
        let (u, v) = if inverse { (u / self.along, v / self.across) } else { (u * self.along, v * self.across) };
        (self.pivot.0 + u * c - v * s, self.pivot.1 + u * s + v * c)
    }
}

pub type Drops = Vec<(&'static str, Drop)>;

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub drops: Drops,
    /// Smoothing distance of the union between body drops.
    pub goo: f32,
    pub warp: Warp,
}

pub const GOO: f32 = 0.9;

impl Scene {
    pub fn new(drops: Drops) -> Scene {
        Scene { drops, goo: GOO, warp: Warp::NONE }
    }

    pub fn get(&self, key: &str) -> Option<Drop> {
        self.drops.iter().find(|(k, _)| *k == key).map(|(_, drop)| *drop)
    }
}

/// Mirrored windows for drops on only one side, so playing a transition
/// backwards is exactly the opposite transition.
const LEAVE: (f32, f32) = (0.0, 0.7);
const ENTER: (f32, f32) = (0.3, 1.0);
/// Fade of a fully melted drop.
const HIDE: f32 = 6.0;
/// Extra goo at the middle of a transition, as a fraction of the rest goo.
const SURGE: f32 = 0.8;

fn mix(x: Drop, y: Drop, t: f32) -> Drop {
    let point = |p: P, q: P| (lerp(p.0, q.0, t), lerp(p.1, q.1, t));
    Drop {
        a: point(x.a, y.a),
        ra: lerp(x.ra, y.ra, t),
        b: point(x.b, y.b),
        rb: lerp(x.rb, y.rb, t),
        layer: x.layer,
        color: mix_color(x.color, y.color, t),
        anchor: point(x.anchor, y.anchor),
        fade: lerp(x.fade, y.fade, t),
        rim: lerp(x.rim, y.rim, t),
    }
}

/// The drop shrunk toward its anchor by `s`, its border thinning with it and
/// its pull on neighbours dissolving as it arrives.
fn melted(drop: Drop, s: f32) -> Drop {
    let gone = Drop { a: drop.anchor, ra: 0.0, b: drop.anchor, rb: 0.0, rim: 0.0, ..drop };
    Drop { fade: drop.fade + HIDE * s.powi(4), ..mix(drop, gone, s) }
}

/// The scene at progress `t` in [0, 1] from `a` to `b`, drops in key order so
/// both directions draw identically.
pub fn blend(a: &Scene, b: &Scene, t: f32) -> Scene {
    let mut keys: Vec<&'static str> = a.drops.iter().chain(&b.drops).map(|(key, _)| *key).collect();
    keys.sort_unstable();
    keys.dedup();
    let drops = keys
        .into_iter()
        .map(|key| {
            let drop = match (a.get(key), b.get(key)) {
                (Some(x), Some(y)) => mix(x, y, ease(t)),
                (Some(x), None) => melted(x, ease(window(t, LEAVE))),
                (None, Some(y)) => melted(y, 1.0 - ease(window(t, ENTER))),
                (None, None) => unreachable!(),
            };
            (key, drop)
        })
        .collect();
    Scene { drops, goo: lerp(a.goo, b.goo, t) * (1.0 + SURGE * (PI * t).sin()), warp: Warp::NONE }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warp_round_trips() {
        let warp = Warp { pivot: (1.0, 2.0), angle: 30.0, along: 1.2, across: 0.8 };
        let (x, y) = warp.apply(warp.apply((3.0, -4.0), false), true);
        assert!((x - 3.0).abs() < 1e-4 && (y + 4.0).abs() < 1e-4);
    }
}
