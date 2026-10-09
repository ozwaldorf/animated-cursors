//! Rasterizes the distance field: colored liquid inside a white border, over
//! a soft shadow. Carved holes are thinner than the border, so they show white.

use cursor_core::color::encode;
use cursor_core::paint::{Shadow, drop_shadow};
use tiny_skia::{Pixmap, Transform};

use crate::field::{BORDER, sample};
use crate::scene::{P, Scene};

const SHADOW: Shadow = Shadow { opacity: 0.35, offset: 1.2, blur: 0.9 };

/// Design-unit box that holds everything the scene can paint.
fn bounds(scene: &Scene) -> (P, P) {
    let margin = BORDER + 4.0 * scene.goo + 1.0;
    let (lo, hi) = scene.drops.iter().fold(((f32::MAX, f32::MAX), (f32::MIN, f32::MIN)), |(lo, hi), (_, d)| {
        let r = d.ra.max(d.rb) + margin;
        let (x0, x1) = (d.a.0.min(d.b.0) - r, d.a.0.max(d.b.0) + r);
        let (y0, y1) = (d.a.1.min(d.b.1) - r, d.a.1.max(d.b.1) + r);
        ((lo.0.min(x0), lo.1.min(y0)), (hi.0.max(x1), hi.1.max(y1)))
    });
    let corners = [lo, (hi.0, lo.1), (lo.0, hi.1), hi].map(|p| scene.warp.apply(p, false));
    corners.iter().fold(((f32::MAX, f32::MAX), (f32::MIN, f32::MIN)), |(lo, hi), &(x, y)| {
        ((lo.0.min(x), lo.1.min(y)), (hi.0.max(x), hi.1.max(y)))
    })
}

/// Composites the scene over `pixmap`; a silhouette paints the rim in black.
fn raster(scene: &Scene, pixmap: &mut Pixmap, transform: Transform, silhouette: bool) {
    let (scale, tx, ty) = (transform.sx, transform.tx, transform.ty);
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let (lo, hi) = bounds(scene);
    let span = |from: f32, to: f32, offset: f32, limit: usize| {
        let pixel = |v: f32| ((v * scale + offset).floor().max(0.0) as usize).min(limit);
        pixel(from)..pixel(to + 1.0 / scale)
    };
    let (columns, rows) = (span(lo.0, hi.0, tx, width), span(lo.1, hi.1, ty, height));
    let data = pixmap.data_mut();
    for y in rows {
        for x in columns.clone() {
            let point = ((x as f32 + 0.5 - tx) / scale, (y as f32 + 0.5 - ty) / scale);
            let sample = sample(scene, point);
            let cover = |distance: f32| (0.5 - distance * scale).clamp(0.0, 1.0);
            let (fill, rim) = (cover(sample.fill), cover(sample.rim));
            if rim == 0.0 {
                continue;
            }
            let source = if silhouette {
                [0.0, 0.0, 0.0, rim]
            } else {
                let white = rim * (1.0 - fill);
                let [r, g, b] = sample.color.map(|c| encode(c) * fill + white);
                [r, g, b, fill + white]
            };
            let pixel = &mut data[(y * width + x) * 4..][..4];
            for (channel, value) in pixel.iter_mut().zip(source) {
                let blended = value * 255.0 + *channel as f32 * (1.0 - source[3]);
                *channel = blended.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

pub fn draw(scene: &Scene, pixmap: &mut Pixmap, transform: Transform) {
    drop_shadow(pixmap, &SHADOW, transform, |pixmap, transform| raster(scene, pixmap, transform, true));
    raster(scene, pixmap, transform, false);
}
