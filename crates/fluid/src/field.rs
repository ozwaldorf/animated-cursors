//! Signed distance field of a scene. Body drops melt together through an
//! exponential smooth union whose weights also blend their colors; carves cut
//! holes out of the result and inlays fill them again.

use cursor_core::color::Color;

use crate::scene::{Drop, Layer, P, Scene};

/// Smoothing of carved and inlaid edges.
const CRISP: f32 = 0.3;

/// Exact distance to a 2D round cone (after Inigo Quilez).
fn cone((px, py): P, drop: &Drop) -> f32 {
    let (a, b, ra, rb) = (drop.a, drop.b, drop.ra, drop.rb);
    let circle = |c: P, r: f32| (px - c.0).hypot(py - c.1) - r;
    let (bx, by) = (b.0 - a.0, b.1 - a.1);
    let l2 = bx * bx + by * by;
    let rr = ra - rb;
    let a2 = l2 - rr * rr;
    if a2 <= 1e-6 {
        return circle(a, ra).min(circle(b, rb));
    }
    let (qx, qy) = (px - a.0, py - a.1);
    let y = qx * bx + qy * by;
    let z = y - l2;
    let (wx, wy) = (qx * l2 - bx * y, qy * l2 - by * y);
    let x2 = wx * wx + wy * wy;
    let (y2, z2) = (y * y * l2, z * z * l2);
    let k = rr.signum() * rr * rr * x2;
    if z.signum() * a2 * z2 > k {
        return (x2 + z2).sqrt() / l2 - rb;
    }
    if y.signum() * a2 * y2 < k {
        return (x2 + y2).sqrt() / l2 - ra;
    }
    ((x2 * a2 / l2).sqrt() + y * rr) / l2 - ra
}

/// Running exponential smooth minimum. Colors blend with squared weights, so
/// hues stay distinct until drops are well merged.
struct Union {
    k: f32,
    min: f32,
    weight: f32,
    tint: f32,
    color: Color,
}

impl Union {
    fn new(k: f32) -> Union {
        Union { k, min: f32::INFINITY, weight: 0.0, tint: 0.0, color: [0.0; 3] }
    }

    fn add(&mut self, distance: f32, color: Color) {
        if distance < self.min {
            let rescale = if self.min.is_finite() { ((distance - self.min) / self.k).exp() } else { 0.0 };
            self.weight *= rescale;
            self.tint *= rescale * rescale;
            self.color = self.color.map(|c| c * rescale * rescale);
            self.min = distance;
        }
        let w = (-(distance - self.min) / self.k).exp();
        self.weight += w;
        self.tint += w * w;
        (0..3).for_each(|i| self.color[i] += w * w * color[i]);
    }

    fn distance(&self) -> f32 {
        self.min - self.k * self.weight.ln()
    }

    fn color(&self) -> Color {
        self.color.map(|c| c / self.tint)
    }
}

/// Width of the white border, in design units.
pub const BORDER: f32 = 1.4;

/// Smooth maximum, for cutting `hole` out of `d`.
fn cut(d: f32, hole: f32) -> f32 {
    let max = d.max(hole);
    max + CRISP * (((d - max) / CRISP).exp() + ((hole - max) / CRISP).exp()).ln()
}

pub struct Sample {
    /// Distance to the liquid.
    pub fill: f32,
    /// Distance to the outside of the border.
    pub rim: f32,
    pub color: Color,
}

/// Distances in design units and color at point `p`. Each drop's border is
/// scaled by its `rim`, so drops melting in or out carry a matching outline.
pub fn sample(scene: &Scene, p: P) -> Sample {
    let warp = scene.warp;
    let p = warp.apply(p, true);
    let layer = |layer: Layer| scene.drops.iter().map(|(_, d)| d).filter(move |d| d.layer == layer);
    let distance = |drop: &Drop| cone(p, drop) + drop.fade;

    let (mut body, mut border) = (Union::new(scene.goo), Union::new(scene.goo));
    for drop in layer(Layer::Body) {
        let d = distance(drop);
        body.add(d, drop.color);
        border.add(d - BORDER * drop.rim, drop.color);
    }
    if body.weight == 0.0 {
        return Sample { fill: f32::INFINITY, rim: f32::INFINITY, color: [0.0; 3] };
    }
    let (mut d, mut outline) = (body.distance(), border.distance());
    let carve = layer(Layer::Carve).map(|drop| (distance(drop), drop.rim)).min_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((hole, rim)) = carve {
        d = cut(d, -hole);
        outline = cut(outline, -hole - BORDER * rim);
    }

    let (mut fill, mut rim) = (Union::new(CRISP), Union::new(CRISP));
    fill.add(d, body.color());
    rim.add(outline, body.color());
    for drop in layer(Layer::Inlay) {
        let d = distance(drop);
        fill.add(d, drop.color);
        rim.add(d - BORDER * drop.rim, drop.color);
    }
    let scale = warp.along.min(warp.across);
    Sample { fill: fill.distance() * scale, rim: rim.distance() * scale, color: fill.color() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Scene, blob, cone as round_cone};

    #[test]
    fn cone_distances() {
        let drop = round_cone((0.0, 0.0), 1.0, (10.0, 0.0), 3.0, [0.0; 3]);
        assert!((cone((-3.0, 0.0), &drop) - 2.0).abs() < 1e-4);
        assert!((cone((15.0, 0.0), &drop) - 2.0).abs() < 1e-4);
        assert!(cone((5.0, 0.0), &drop) < 0.0);
        let circle = blob((0.0, 0.0), 2.0, [0.0; 3]);
        assert!((cone((5.0, 0.0), &circle) - 3.0).abs() < 1e-4);
    }

    #[test]
    fn lone_drop_is_exact() {
        let scene = Scene::new(vec![("body", blob((0.0, 0.0), 4.0, [1.0, 0.0, 0.0]))]);
        let sample = sample(&scene, (0.0, 6.0));
        assert!((sample.fill - 2.0).abs() < 1e-4, "{}", sample.fill);
        assert!((sample.rim - 2.0 + BORDER).abs() < 1e-4, "{}", sample.rim);
        assert!((sample.color[0] - 1.0).abs() < 1e-4);
    }
}
