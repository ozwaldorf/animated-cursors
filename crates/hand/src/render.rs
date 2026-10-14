//! Ray marches the solids from the front and shades them in flat bands with
//! a specular glint, in shadow wherever another part blocks the light. Ink
//! traces the silhouette, edges that pass in front of another surface, and
//! seams where parts meet. The hand casts a drop shadow on a backdrop behind
//! it.

use cursor_core::Palette;
use cursor_core::color::{Color, encode, hex, mix};
use cursor_core::motion::lerp;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

use crate::rig::{Scene, model};
use crate::solid::{Model, Solid};
use crate::vector::V;

/// Ink widths in design units.
const INK: f32 = 0.7;
const SEAM: f32 = 0.4;
/// Drop shadow: depth of the backdrop it falls on, how far it shifts per
/// unit of height above it, how its edge widens with that height, and how
/// dark it is.
const FLOOR: f32 = -10.0;
const SLANT: f32 = 0.12;
const PENUMBRA: f32 = 0.025;
const SHADOW_OPACITY: f32 = 0.35;
/// Edge width of shadows cast on the hand, kept crisp like the shading.
const CAST: f32 = 0.01;
const TILE: usize = 8;
const STEPS: usize = 112;
const SURFACE: f32 = 2e-3;
/// How strongly creases fall into shadow.
const OCCLUSION: f32 = 0.7;

fn light() -> V {
    V(-0.45, -0.7, 0.6).normalized()
}

/// Light of the drop shadow, from the same side but steeper, so the shadow
/// falls close behind the hand.
fn backlight() -> V {
    let l = light();
    V(l.0, l.1, l.0.hypot(l.1) / SLANT).normalized()
}

/// Share of the light reaching `p` from `dir`, marched until the ray passes
/// `front`, its edge widening by `softness` per unit of distance from what
/// blocks it.
fn visibility(model: &Model, subset: &[u16], p: V, dir: V, front: f32, softness: f32) -> f32 {
    let (mut t, mut lit) = (0.05, 1.0f32);
    for _ in 0..STEPS {
        let q = p + dir * t;
        if q.2 > front {
            break;
        }
        let d = model.field(subset, q).0;
        if d < SURFACE {
            return 0.0;
        }
        lit = lit.min(d / (softness * t));
        t += d.max(0.01);
    }
    lit.clamp(0.0, 1.0)
}

/// How enclosed the surface at `p` facing `n` is, from 0 (open) to 1, by
/// probing the field at a few steps outward.
fn occlusion(model: &Model, subset: &[u16], p: V, n: V) -> f32 {
    let (mut occluded, mut weight) = (0.0, 1.0);
    for step in 1..=5 {
        let h = 0.22 * step as f32;
        occluded += (h - model.field(subset, p + n * h).0).max(0.0) * weight;
        weight *= 0.7;
    }
    (OCCLUSION * occluded).clamp(0.0, 1.0)
}

/// Tone of a surface facing `n`, darkened by `occluded`: a glint, the lit
/// band, or the shaded band, which also covers it where it is not `visible`.
fn tone((lit_tone, shaded): (Color, Color), n: V, occluded: f32, visible: impl Fn() -> bool) -> Color {
    let lit = n.dot(light()) - occluded;
    let glint = n.dot((light() + V::Z).normalized()) - occluded;
    match () {
        _ if lit <= 0.38 || !visible() => shaded,
        _ if glint > 0.97 => mix(lit_tone, hex(0xffffff), 0.75),
        _ => lit_tone,
    }
}

/// The solids near a tile and the wider set that could shade it.
struct Reach {
    near: Vec<u16>,
    wide: Vec<u16>,
    /// Depth span of the near solids, and the front of every solid.
    depth: (f32, f32),
    front: f32,
}

/// Premultiplied sRGB color of the ray through (`x`, `y`); `pixel` is the
/// sample's width in design units.
fn trace(colors: &Palette, model: &Model, reach: &Reach, (x, y): (f32, f32), pixel: f32) -> [f32; 4] {
    let (subset, (back, front), ink) = (&reach.near[..], reach.depth, colors["ink"]);
    let cover = |distance: f32| (0.5 - distance / pixel).clamp(0.0, 1.0);
    let mut z = front;
    // Closest approach so far, the lowest dip since the ray last pulled away
    // from a surface, and the closest edge it passed on the way.
    let (mut near, mut low, mut graze) = (f32::INFINITY, f32::INFINITY, f32::INFINITY);
    for _ in 0..if subset.is_empty() { 0 } else { STEPS } {
        let p = V(x, y, z);
        let (d, index) = model.field(subset, p);
        near = near.min(d);
        if d < SURFACE {
            let solid = &model.solids[index];
            let seam = subset
                .iter()
                .map(|&j| &model.solids[j as usize])
                .filter(|other| model.seams(solid, other))
                .map(|other| other.distance(p))
                .fold(f32::INFINITY, f32::min);
            let line = cover(graze - INK).max(cover(seam - SEAM));
            let n = model.normal(subset, p);
            let start = p + n * (4.0 * SURFACE);
            let visible = || visibility(model, &reach.wide, start, light(), reach.front, CAST) > 0.5;
            let color = tone(model.tones(subset, index, p), n, occlusion(model, subset, p, n), visible);
            let [r, g, b] = std::array::from_fn(|i| encode(lerp(color[i], ink[i], line)));
            return [r, g, b, 1.0];
        }
        if d < low {
            low = d;
        } else if d > low + INK {
            graze = graze.min(low);
            low = d;
        }
        z -= d.max(0.01);
        if z < back {
            break;
        }
    }
    // Ink along the silhouette, over the shadow on the backdrop.
    let alpha = cover(near - INK);
    let lit = visibility(model, &reach.wide, V(x, y, FLOOR), backlight(), reach.front, PENUMBRA);
    let shade = SHADOW_OPACITY * (1.0 - lit) * (1.0 - alpha);
    let paint = |i: usize| encode(ink[i]) * alpha + encode(colors["shadow"][i]) * shade;
    [paint(0), paint(1), paint(2), alpha + shade]
}

/// Samples per pixel along each axis at `scale` pixels per design unit.
fn samples(scale: f32) -> usize {
    match scale {
        s if s >= 4.0 => 1,
        s if s >= 2.0 => 2,
        _ => 3,
    }
}

fn raster(colors: &Palette, model: &Model, pixmap: &mut Pixmap, transform: Transform) {
    let (scale, tx, ty) = (transform.sx, transform.tx, transform.ty);
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let bounds: Vec<(V, f32)> = model.solids.iter().map(Solid::bound).collect();
    let reach = INK + 0.6;
    let (back, front) = bounds
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(back, front), &(c, r)| (back.min(c.2 - r), front.max(c.2 + r)));
    // How far across the screen a light ray drifts while rising by `rise`.
    let drift = |dir: V, rise: f32| (dir.0 / dir.2 * rise, dir.1 / dir.2 * rise);
    let drifts = [drift(light(), front - back), drift(backlight(), front - FLOOR)];
    let n = samples(scale);
    let data = pixmap.data_mut();
    for top in (0..height).step_by(TILE) {
        for left in (0..width).step_by(TILE) {
            let unit = |p: usize, offset: f32| (p as f32 - offset) / scale;
            let (x0, x1, y0, y1) = (unit(left, tx), unit(left + TILE, tx), unit(top, ty), unit(top + TILE, ty));
            let (wx0, wy0, wx1, wy1) = drifts.iter().fold((x0, y0, x1, y1), |(a, b, c, d), &(dx, dy)| {
                (a.min(x0 + dx), b.min(y0 + dy), c.max(x1 + dx), d.max(y1 + dy))
            });
            let wide = model.near((wx0, wy0), (wx1, wy1), reach);
            if wide.is_empty() {
                continue;
            }
            let near = model.near((x0, y0), (x1, y1), reach);
            let depth = near.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(back, front), &i| {
                let (c, r) = bounds[i as usize];
                (back.min(c.2 - r - reach), front.max(c.2 + r + reach))
            });
            let tile = Reach { near, wide, depth, front };
            for py in top..(top + TILE).min(height) {
                for px in left..(left + TILE).min(width) {
                    let mut sum = [0.0f32; 4];
                    for s in 0..n * n {
                        let at =
                            |p: usize, k: usize, offset: f32| (p as f32 + (k as f32 + 0.5) / n as f32 - offset) / scale;
                        let point = (at(px, s % n, tx), at(py, s / n, ty));
                        let color = trace(colors, model, &tile, point, 1.0 / (scale * n as f32));
                        for (total, c) in sum.iter_mut().zip(color) {
                            *total += c;
                        }
                    }
                    let pixel = &mut data[(py * width + px) * 4..][..4];
                    for (channel, value) in pixel.iter_mut().zip(sum) {
                        *channel = (value / (n * n) as f32 * 255.0).round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
    }
}

pub fn draw(colors: &Palette, scene: &Scene, pixmap: &mut Pixmap, transform: Transform) {
    let render = |scene: &Scene| {
        let mut image = Pixmap::new(pixmap.width(), pixmap.height()).expect("nonzero size");
        raster(colors, &model(colors, scene), &mut image, transform);
        image
    };
    let mut image = render(scene);
    // A fading prop: cross-fade to the scene without it.
    let fade = scene.props.iter().map(|(_, p)| p.fade).fold(1.0, f32::min);
    if fade < 1.0 {
        let props = scene.props.iter().filter(|(_, p)| p.fade >= 1.0).cloned().collect();
        let bare = render(&Scene { props, ..scene.clone() });
        for (pixel, under) in image.data_mut().iter_mut().zip(bare.data()) {
            *pixel = (*under as f32 + (*pixel as f32 - *under as f32) * fade).round() as u8;
        }
    }
    pixmap.draw_pixmap(0, 0, image.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
}
