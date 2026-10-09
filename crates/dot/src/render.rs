//! Rasterizes rig parts. Every part is a stroked centerline, so the halo is
//! the same geometry stroked wider and overlapping parts merge into one outline.

use cursor_core::motion::polar;
use cursor_core::paint::{Shadow, drop_shadow};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

use crate::rig::{Kind, Part};

const INK: [u8; 3] = [0x12, 0x12, 0x12];
const HALO: [u8; 3] = [0xff, 0xff, 0xff];
const BORDER: f32 = 1.5;
const SHADOW: Shadow = Shadow { opacity: 0.35, offset: 1.2, blur: 0.9 };

enum Geometry {
    Point(f32, f32),
    Line(Path),
}

fn arc(x: f32, y: f32, radius: f32, start: f32, sweep: f32) -> Option<Path> {
    if sweep >= 359.99 {
        return PathBuilder::from_circle(x, y, radius);
    }
    let segments = (sweep / 90.0).ceil().max(1.0) as usize;
    let step = sweep / segments as f32;
    let k = 4.0 / 3.0 * (step.to_radians() / 4.0).tan() * radius;
    let mut path = PathBuilder::new();
    let (px, py) = polar(x, y, radius, start);
    path.move_to(px, py);
    for index in 0..segments {
        let (a, b) = (start + step * index as f32, start + step * (index + 1) as f32);
        let (sa, ca) = a.to_radians().sin_cos();
        let (sb, cb) = b.to_radians().sin_cos();
        let (p0, p3) = (polar(x, y, radius, a), polar(x, y, radius, b));
        path.cubic_to(p0.0 - k * sa, p0.1 + k * ca, p3.0 + k * sb, p3.1 - k * cb, p3.0, p3.1);
    }
    path.finish()
}

fn geometry(part: &Part) -> Option<Geometry> {
    let point = Some(Geometry::Point(part.x, part.y));
    match part.kind {
        Kind::Bar { length, angle } => {
            let half = (length - part.width).max(0.0) / 2.0;
            if half < 1e-4 {
                return point;
            }
            let angle = angle.unwrap_or(0.0);
            let (start, end) = (polar(part.x, part.y, half, angle + 180.0), polar(part.x, part.y, half, angle));
            let mut path = PathBuilder::new();
            path.move_to(start.0, start.1);
            path.line_to(end.0, end.1);
            path.finish().map(Geometry::Line)
        }
        Kind::Chevron { angle, arm } => {
            if arm < 1e-4 {
                return point;
            }
            let ends = [45.0, -45.0].map(|turn| polar(part.x, part.y, arm, angle + 180.0 + turn));
            let mut path = PathBuilder::new();
            path.move_to(ends[0].0, ends[0].1);
            path.line_to(part.x, part.y);
            path.line_to(ends[1].0, ends[1].1);
            path.finish().map(Geometry::Line)
        }
        Kind::Arc { radius, start, sweep, .. } => {
            if radius < 1e-4 {
                return point;
            }
            if sweep < 1e-2 {
                let (x, y) = polar(part.x, part.y, radius, start);
                return Some(Geometry::Point(x, y));
            }
            arc(part.x, part.y, radius, start, sweep).map(Geometry::Line)
        }
    }
}

/// Draws parts in one color; `grow` widens every stroke by that much per side,
/// tapering for parts thinner than two units so collapsing parts vanish.
fn layer(pixmap: &mut Pixmap, parts: &[(&str, Part)], [r, g, b]: [u8; 3], grow: f32, transform: Transform) {
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, 255);
    for (_, part) in parts.iter().filter(|(_, part)| part.width > 0.0) {
        let width = part.width + 2.0 * grow * (part.width / 2.0).min(1.0);
        match geometry(part) {
            Some(Geometry::Point(x, y)) => {
                if let Some(circle) = PathBuilder::from_circle(x, y, width / 2.0) {
                    pixmap.fill_path(&circle, &paint, FillRule::Winding, transform, None);
                }
            }
            Some(Geometry::Line(path)) => {
                let stroke =
                    Stroke { width, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Stroke::default() };
                pixmap.stroke_path(&path, &paint, &stroke, transform, None);
            }
            None => {}
        }
    }
}

/// Draws parts with a white border and soft shadow.
pub fn draw(parts: &[(&str, Part)], pixmap: &mut Pixmap, transform: Transform) {
    drop_shadow(pixmap, &SHADOW, transform, |pixmap, transform| layer(pixmap, parts, [0, 0, 0], BORDER, transform));
    layer(pixmap, parts, HALO, BORDER, transform);
    layer(pixmap, parts, INK, 0.0, transform);
}
