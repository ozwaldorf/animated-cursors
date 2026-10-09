//! Scenes of colored shapes and glyph strokes, and the morph between them.

use cursor_core::motion::{ease, lerp, spring, window};

use crate::outline::{Outline, P};

/// Linear-light sRGB color.
pub type Color = [f32; 3];

pub fn hex(rgb: u32) -> Color {
    let channel = |shift: u32| {
        let c = ((rgb >> shift) & 0xff) as f32 / 255.0;
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    [channel(16), channel(8), channel(0)]
}

// Published OKLab matrices, kept at full precision.
#[allow(clippy::excessive_precision)]
fn oklab([r, g, b]: Color) -> Color {
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

#[allow(clippy::excessive_precision)]
fn linear([l, a, b]: Color) -> Color {
    let l_ = (l + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m_ = (l - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s_ = (l - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    [
        4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
        -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
        -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_,
    ]
    .map(|c| c.clamp(0.0, 1.0))
}

/// Perceptual blend, so hues sweep instead of turning muddy.
pub fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let (a, b) = (oklab(a), oklab(b));
    linear(std::array::from_fn(|i| lerp(a[i], b[i], t)))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Blob {
    pub center: P,
    pub outline: Outline,
    pub rotation: f32,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    /// Glyphs sharing a key and point count morph into each other.
    pub key: &'static str,
    pub points: Vec<P>,
    pub width: f32,
    pub color: Color,
    /// Grows and shrinks about `pivot` as glyphs come and go.
    pub scale: f32,
    pub pivot: P,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub blobs: Vec<Blob>,
    pub glyphs: Vec<Glyph>,
}

/// Mirrored windows for parts on only one side, so playing a transition
/// backwards reads as the opposite transition.
const LEAVE: (f32, f32) = (0.0, 0.55);
const ENTER: (f32, f32) = (0.45, 1.0);
/// Extra spin at the middle of a morph, in degrees.
const TWIST: f32 = 30.0;

fn grown(blob: &Blob, k: f32) -> Blob {
    Blob { outline: blob.outline.scaled(k.max(0.0)), ..blob.clone() }
}

fn sized(glyph: &Glyph, k: f32) -> Glyph {
    Glyph { scale: glyph.scale * k.max(0.0), ..glyph.clone() }
}

/// Morphs blob by blob in order. Glyphs with a counterpart morph; the rest
/// shrink away or pop in through their pivots.
pub fn blend(a: &Scene, b: &Scene, t: f32) -> Scene {
    let shape = spring(t);
    let hue = ease(t);
    let twist = TWIST * (std::f32::consts::PI * t).sin();
    let count = a.blobs.len().max(b.blobs.len());
    let blobs = (0..count)
        .map(|i| match (a.blobs.get(i), b.blobs.get(i)) {
            (Some(x), Some(y)) => Blob {
                center: (lerp(x.center.0, y.center.0, shape), lerp(x.center.1, y.center.1, shape)),
                outline: x.outline.mix(&y.outline, shape),
                rotation: lerp(x.rotation, y.rotation, shape) + twist,
                color: mix_color(x.color, y.color, hue),
            },
            (Some(x), None) => grown(x, 1.0 - ease(window(t, LEAVE))),
            (None, Some(y)) => grown(y, spring(window(t, ENTER))),
            (None, None) => unreachable!(),
        })
        .collect();
    let partner = |glyph: &Glyph, others: &[Glyph]| {
        others.iter().find(|o| o.key == glyph.key && o.points.len() == glyph.points.len()).cloned()
    };
    let shared = a.glyphs.iter().filter_map(|x| {
        let y = partner(x, &b.glyphs)?;
        let points = x.points.iter().zip(&y.points).map(|(p, q)| (lerp(p.0, q.0, shape), lerp(p.1, q.1, shape)));
        Some(Glyph {
            points: points.collect(),
            width: lerp(x.width, y.width, shape),
            color: mix_color(x.color, y.color, hue),
            scale: lerp(x.scale, y.scale, shape),
            pivot: (lerp(x.pivot.0, y.pivot.0, shape), lerp(x.pivot.1, y.pivot.1, shape)),
            key: x.key,
        })
    });
    let leaving = a.glyphs.iter().filter(|g| partner(g, &b.glyphs).is_none());
    let entering = b.glyphs.iter().filter(|g| partner(g, &a.glyphs).is_none());
    let glyphs = shared
        .chain(leaving.map(|g| sized(g, 1.0 - ease(window(t, LEAVE)))))
        .chain(entering.map(|g| sized(g, spring(window(t, ENTER)))))
        .collect();
    Scene { blobs, glyphs }
}

impl Scene {
    /// Turns the whole scene by `degrees` about `pivot` and scales it there.
    pub fn about(self, pivot: P, degrees: f32, scale: f32) -> Scene {
        let (s, c) = degrees.to_radians().sin_cos();
        let map = |(x, y): P| {
            let (dx, dy) = (x - pivot.0, y - pivot.1);
            (pivot.0 + (dx * c - dy * s) * scale, pivot.1 + (dx * s + dy * c) * scale)
        };
        Scene {
            blobs: self
                .blobs
                .into_iter()
                .map(|b| Blob {
                    center: map(b.center),
                    outline: b.outline.scaled(scale),
                    rotation: b.rotation + degrees,
                    ..b
                })
                .collect(),
            glyphs: self
                .glyphs
                .into_iter()
                .map(|g| Glyph {
                    points: g.points.into_iter().map(map).collect(),
                    pivot: map(g.pivot),
                    width: g.width * scale,
                    ..g
                })
                .collect(),
        }
    }

    /// Spins every shape in place, leaving glyphs alone.
    pub fn spin(mut self, degrees: f32) -> Scene {
        self.blobs.iter_mut().for_each(|b| b.rotation += degrees);
        self
    }

    pub fn shift(self, (dx, dy): P) -> Scene {
        let mut scene = self;
        scene.blobs.iter_mut().for_each(|b| b.center = (b.center.0 + dx, b.center.1 + dy));
        for g in &mut scene.glyphs {
            g.points.iter_mut().for_each(|p| *p = (p.0 + dx, p.1 + dy));
            g.pivot = (g.pivot.0 + dx, g.pivot.1 + dy);
        }
        scene
    }

    /// Moves off-center glyphs `distance` further from the origin.
    pub fn push_glyphs(mut self, distance: f32) -> Scene {
        for g in &mut self.glyphs {
            let length = g.pivot.0.hypot(g.pivot.1);
            if length > 0.1 {
                let (dx, dy) = (g.pivot.0 / length * distance, g.pivot.1 / length * distance);
                g.points.iter_mut().for_each(|p| *p = (p.0 + dx, p.1 + dy));
                g.pivot = (g.pivot.0 + dx, g.pivot.1 + dy);
            }
        }
        self
    }

    /// Moves off-center shapes `distance` further from the origin.
    pub fn push_blobs(mut self, distance: f32) -> Scene {
        for b in &mut self.blobs {
            let length = b.center.0.hypot(b.center.1);
            if length > 0.1 {
                b.center = (b.center.0 + b.center.0 / length * distance, b.center.1 + b.center.1 / length * distance);
            }
        }
        self
    }

    pub fn pulse_glyphs(mut self, scale: f32) -> Scene {
        self.glyphs.iter_mut().for_each(|g| g.scale *= scale);
        self
    }
}
