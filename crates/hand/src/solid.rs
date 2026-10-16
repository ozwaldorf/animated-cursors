//! Solids as signed distance functions, and the field of a set of them.
//! Solids of one body melt together; bodies only touch.

use cursor_core::color::{Color, mix};

use crate::vector::{Rot, V};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Form {
    /// Round cone between spheres of radius `ra` at `a` and `rb` at `b`.
    Cone { a: V, ra: f32, b: V, rb: f32 },
    /// Box of half extents `half`, its edges rounded by `round`.
    Slab { half: V, round: f32 },
    /// Torus of `major` radius about the local y axis.
    Ring { major: f32, minor: f32 },
    /// Cone along local y from radius `r0` at `-half` to `r1` at `half`, its
    /// edges rounded by `round`.
    Frustum { half: f32, r0: f32, r1: f32, round: f32 },
    /// Ellipsoid with semi-axes `radii`.
    Ellipsoid { radii: V },
}

pub const BODIES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Solid {
    pub form: Form,
    /// Placement of the form's local frame; cones ignore it.
    pub at: V,
    pub rot: Rot,
    /// Lit and shaded tones.
    pub light: Color,
    pub dark: Color,
    /// Ink seams are drawn where parts meet, unless one joins the other.
    pub part: u8,
    pub joins: u8,
    /// Index below `BODIES`, and the smoothing of its union.
    pub body: u8,
    pub melt: f32,
}

fn round_cone(p: V, a: V, ra: f32, b: V, rb: f32) -> f32 {
    let ba = b - a;
    let l2 = ba.dot(ba);
    let rr = ra - rb;
    let a2 = l2 - rr * rr;
    if a2 <= 1e-6 {
        return ((p - a).length() - ra).min((p - b).length() - rb);
    }
    let pa = p - a;
    let y = pa.dot(ba);
    let z = y - l2;
    let w = pa * l2 - ba * y;
    let x2 = w.dot(w);
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

/// Capped cone (after Inigo Quilez).
fn frustum(p: V, half: f32, r0: f32, r1: f32) -> f32 {
    let (qx, qy) = (p.0.hypot(p.2), p.1);
    let (k1x, k1y) = (r1, half);
    let (k2x, k2y) = (r1 - r0, 2.0 * half);
    let cax = qx - qx.min(if qy < 0.0 { r0 } else { r1 });
    let cay = qy.abs() - half;
    let s = ((k1x - qx) * k2x + (k1y - qy) * k2y) / (k2x * k2x + k2y * k2y);
    let s = s.clamp(0.0, 1.0);
    let (cbx, cby) = (qx - k1x + k2x * s, qy - k1y + k2y * s);
    let sign = if cbx < 0.0 && cay < 0.0 { -1.0 } else { 1.0 };
    sign * (cax * cax + cay * cay).min(cbx * cbx + cby * cby).sqrt()
}

/// Smooth minimum with polynomial blending over `k`.
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = (k - (a - b).abs()).max(0.0) / k.max(1e-6);
    a.min(b) - h * h * k / 4.0
}

impl Solid {
    pub fn distance(&self, p: V) -> f32 {
        if let Form::Cone { a, ra, b, rb } = self.form {
            return round_cone(p, a, ra, b, rb);
        }
        let q = self.rot.undo(p - self.at);
        match self.form {
            Form::Slab { half, round } => {
                let d = q.abs() - half + V(round, round, round);
                d.max(0.0).length() + d.0.max(d.1).max(d.2).min(0.0) - round
            }
            Form::Ring { major, minor } => (q.0.hypot(q.2) - major).hypot(q.1) - minor,
            // A close bound rather than the exact distance (after Inigo Quilez).
            Form::Ellipsoid { radii: r } => {
                let k0 = V(q.0 / r.0, q.1 / r.1, q.2 / r.2).length();
                let k1 = V(q.0 / (r.0 * r.0), q.1 / (r.1 * r.1), q.2 / (r.2 * r.2)).length();
                if k1 > 0.0 { k0 * (k0 - 1.0) / k1 } else { -r.0.min(r.1).min(r.2) }
            }
            Form::Frustum { half, r0, r1, round } => {
                frustum(q, half - round, (r0 - round).max(0.0), (r1 - round).max(0.0)) - round
            }
            Form::Cone { .. } => unreachable!(),
        }
    }

    /// Center and radius of a sphere holding the solid.
    pub fn bound(&self) -> (V, f32) {
        match self.form {
            Form::Cone { a, ra, b, rb } => ((a + b) * 0.5, (a - b).length() / 2.0 + ra.max(rb)),
            Form::Slab { half, .. } => (self.at, half.length()),
            Form::Ring { major, minor } => (self.at, major + minor),
            Form::Frustum { half, r0, r1, .. } => (self.at, half.hypot(r0.max(r1))),
            Form::Ellipsoid { radii } => (self.at, radii.0.max(radii.1).max(radii.2)),
        }
    }

    /// Whether an ink seam runs where this solid meets `other`.
    pub fn seams(&self, other: &Solid) -> bool {
        self.part != other.part && self.joins != other.part && other.joins != self.part
    }
}

/// Two bodies drawn as one shape partway between them: `to` by `t`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blend {
    pub from: u8,
    pub to: u8,
    pub t: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub solids: Vec<Solid>,
    pub blends: Vec<Blend>,
}

impl Model {
    /// The blend a body takes part in, and the weight of that body in it.
    fn partner(&self, body: u8) -> Option<(u8, f32)> {
        self.blends.iter().find_map(|b| match body {
            _ if body == b.from => Some((b.to, 1.0 - b.t)),
            _ if body == b.to => Some((b.from, b.t)),
            _ => None,
        })
    }

    /// Indices of every solid that could reach within `reach` of the box
    /// from (`x0`, `y0`) to (`x1`, `y1`), with whole blends kept together.
    pub fn near(&self, (x0, y0): (f32, f32), (x1, y1): (f32, f32), reach: f32) -> Vec<u16> {
        let all: Vec<u16> = (0..self.solids.len() as u16).collect();
        self.keep(&all, |(c, r)| {
            let dx = (x0 - c.0).max(c.0 - x1).max(0.0);
            let dy = (y0 - c.1).max(c.1 - y1).max(0.0);
            dx.hypot(dy) <= r + reach
        })
    }

    /// The solids of `subset` whose bounding sphere passes `close`, with
    /// whole blends kept together.
    pub fn keep(&self, subset: &[u16], close: impl Fn((V, f32)) -> bool) -> Vec<u16> {
        let mut hit = [false; BODIES];
        let close: Vec<bool> = subset
            .iter()
            .map(|&i| {
                let solid = &self.solids[i as usize];
                let close = close(solid.bound());
                hit[solid.body as usize] |= close;
                close
            })
            .collect();
        for b in &self.blends {
            let both = hit[b.from as usize] || hit[b.to as usize];
            (hit[b.from as usize], hit[b.to as usize]) = (both, both);
        }
        let blended = |body: u8| self.partner(body).is_some() && hit[body as usize];
        subset
            .iter()
            .zip(close)
            .filter(|&(&i, close)| close || blended(self.solids[i as usize].body))
            .map(|(&i, _)| i)
            .collect()
    }

    /// Distance to the nearest surface at `p` and the index of the solid
    /// closest to it, over the solids in `subset`.
    pub fn field(&self, subset: &[u16], p: V) -> (f32, usize) {
        let mut bodies = [f32::INFINITY; BODIES];
        let (mut best, mut nearest) = (f32::INFINITY, 0);
        for &index in subset {
            let solid = &self.solids[index as usize];
            let d = solid.distance(p);
            if d < best {
                (best, nearest) = (d, index as usize);
            }
            let body = &mut bodies[solid.body as usize];
            *body = smin(*body, d, solid.melt);
        }
        for b in &self.blends {
            let (from, to) = (bodies[b.from as usize], bodies[b.to as usize]);
            if from.is_finite() && to.is_finite() {
                (bodies[b.from as usize], bodies[b.to as usize]) = (from + (to - from) * b.t, f32::INFINITY);
            }
        }
        (bodies.into_iter().fold(f32::INFINITY, f32::min), nearest)
    }

    /// Surface normal at `p` from the field's gradient.
    pub fn normal(&self, subset: &[u16], p: V) -> V {
        const H: f32 = 0.01;
        let taps = [V(1.0, -1.0, -1.0), V(-1.0, -1.0, 1.0), V(-1.0, 1.0, -1.0), V(1.0, 1.0, 1.0)];
        taps.into_iter().fold(V::default(), |sum, tap| sum + tap * self.field(subset, p + tap * H).0).normalized()
    }

    /// Lit and shaded tones at `p` on solid `index`, mixed with the nearest
    /// solid of the body it blends with.
    pub fn tones(&self, subset: &[u16], index: usize, p: V) -> (Color, Color) {
        let solid = &self.solids[index];
        let Some((other, weight)) = self.partner(solid.body) else {
            return (solid.light, solid.dark);
        };
        let nearest = subset
            .iter()
            .map(|&i| &self.solids[i as usize])
            .filter(|s| s.body == other)
            .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)));
        match nearest {
            Some(o) => (mix(o.light, solid.light, weight), mix(o.dark, solid.dark, weight)),
            None => (solid.light, solid.dark),
        }
    }

    /// Whether an ink seam runs where `a` meets `b`; never inside a blend.
    pub fn seams(&self, a: &Solid, b: &Solid) -> bool {
        a.seams(b) && self.partner(a.body).is_none_or(|(other, _)| other != b.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(form: Form) -> Solid {
        Solid {
            form,
            at: V::default(),
            rot: Rot::IDENTITY,
            light: [1.0; 3],
            dark: [0.5; 3],
            part: 0,
            joins: 0,
            body: 0,
            melt: 0.1,
        }
    }

    #[test]
    fn distances() {
        let cone = solid(Form::Cone { a: V::default(), ra: 1.0, b: V(10.0, 0.0, 0.0), rb: 3.0 });
        assert!((cone.distance(V(-3.0, 0.0, 0.0)) - 2.0).abs() < 1e-4);
        assert!((cone.distance(V(10.0, 0.0, 5.0)) - 2.0).abs() < 1e-4);
        let slab = solid(Form::Slab { half: V(2.0, 1.0, 1.0), round: 0.5 });
        assert!((slab.distance(V(5.0, 0.0, 0.0)) - 3.0).abs() < 1e-4);
        assert!((slab.distance(V(0.0, 0.0, -2.0)) - 1.0).abs() < 1e-4);
        let ring = solid(Form::Ring { major: 4.0, minor: 1.0 });
        assert!((ring.distance(V(0.0, 0.0, 0.0)) - 3.0).abs() < 1e-4);
        assert!((ring.distance(V(4.0, 3.0, 0.0)) - 2.0).abs() < 1e-4);
        let ellipsoid = solid(Form::Ellipsoid { radii: V(3.0, 2.0, 1.0) });
        assert!((ellipsoid.distance(V(5.0, 0.0, 0.0)) - 2.0).abs() < 1e-4);
        assert!((ellipsoid.distance(V(0.0, 0.0, 3.0)) - 2.0).abs() < 1e-4);
        assert!(ellipsoid.distance(V::default()) < 0.0);
        let frustum = solid(Form::Frustum { half: 2.0, r0: 3.0, r1: 1.0, round: 0.0 });
        assert!((frustum.distance(V(0.0, 5.0, 0.0)) - 3.0).abs() < 1e-4);
        assert!((frustum.distance(V(0.0, -2.5, 1.0)) - 0.5).abs() < 1e-4);
        assert!(frustum.distance(V::default()) < 0.0);
    }

    #[test]
    fn blends_lerp_between_bodies() {
        let a = solid(Form::Cone { a: V::default(), ra: 1.0, b: V::default(), rb: 1.0 });
        let b = Solid { body: 1, part: 1, ..solid(Form::Cone { a: V::default(), ra: 3.0, b: V::default(), rb: 3.0 }) };
        let model = Model { solids: vec![a, b], blends: vec![Blend { from: 0, to: 1, t: 0.25 }] };
        assert!((model.field(&[0, 1], V(5.0, 0.0, 0.0)).0 - 3.5).abs() < 1e-4);
        assert!(!model.seams(&model.solids[0], &model.solids[1]));
    }

    #[test]
    fn bodies_touch_without_melting() {
        let mut a = solid(Form::Cone { a: V::default(), ra: 1.0, b: V::default(), rb: 1.0 });
        let mut b = solid(Form::Cone { a: V(3.0, 0.0, 0.0), ra: 1.0, b: V(3.0, 0.0, 0.0), rb: 1.0 });
        a.melt = 2.0;
        b.melt = 2.0;
        let midpoint = V(1.5, 0.0, 0.0);
        let model = |a, b| Model { solids: vec![a, b], blends: vec![] };
        assert!(model(a, b).field(&[0, 1], midpoint).0 < 0.5 - 1e-3);
        b.body = 1;
        assert!((model(a, b).field(&[0, 1], midpoint).0 - 0.5).abs() < 1e-4);
    }
}
