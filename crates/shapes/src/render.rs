//! Paints shapes with a white rim and soft shadow, then their glyphs.

use cursor_core::color::{Color, encode};
use cursor_core::paint::{Shadow, drop_shadow};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

use crate::outline::P;
use crate::scene::Scene;

const BORDER: f32 = 1.4;
const SHADOW: Shadow = Shadow { opacity: 0.35, offset: 1.2, blur: 0.9 };

fn path(points: &[P], closed: bool) -> Option<Path> {
    let mut builder = PathBuilder::new();
    builder.move_to(points.first()?.0, points[0].1);
    points[1..].iter().for_each(|&(x, y)| builder.line_to(x, y));
    if closed {
        builder.close();
    }
    builder.finish()
}

fn paint(color: Color) -> Paint<'static> {
    let [r, g, b] = color.map(|c| (encode(c) * 255.0).round() as u8);
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, 255);
    paint
}

fn stroke(width: f32) -> Stroke {
    Stroke { width, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Stroke::default() }
}

/// Fills a shape and, when `width` is positive, strokes it too.
fn solid(pixmap: &mut Pixmap, path: &Path, color: Color, width: f32, transform: Transform) {
    let paint = paint(color);
    pixmap.fill_path(path, &paint, FillRule::Winding, transform, None);
    if width > 0.0 {
        pixmap.stroke_path(path, &paint, &stroke(width), transform, None);
    }
}

pub fn draw(scene: &Scene, pixmap: &mut Pixmap, transform: Transform) {
    let shapes: Vec<(Path, Color)> = scene
        .blobs
        .iter()
        .filter(|blob| blob.outline.0.iter().any(|&r| r > 0.01))
        .filter_map(|blob| Some((path(&blob.outline.points(blob.center, blob.rotation), true)?, blob.color)))
        .collect();
    drop_shadow(pixmap, &SHADOW, transform, |pixmap, transform| {
        shapes.iter().for_each(|(path, _)| solid(pixmap, path, [0.0; 3], BORDER * 2.0, transform))
    });
    shapes.iter().for_each(|(path, _)| solid(pixmap, path, [1.0; 3], BORDER * 2.0, transform));
    shapes.iter().for_each(|(path, color)| solid(pixmap, path, *color, 0.0, transform));
    for glyph in scene.glyphs.iter().filter(|glyph| glyph.scale > 0.02) {
        let k = glyph.scale;
        let (px, py) = glyph.pivot;
        let points: Vec<P> = glyph.points.iter().map(|&(x, y)| (px + (x - px) * k, py + (y - py) * k)).collect();
        if let Some(path) = path(&points, false) {
            pixmap.stroke_path(&path, &paint(glyph.color), &stroke(glyph.width * k), transform, None);
        }
    }
}
