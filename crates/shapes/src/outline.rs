//! Material 3 Expressive shapes as polar outlines: radii sampled at evenly
//! spaced angles around a center. Any two outlines morph by interpolating
//! their radii, which is smooth for the star-shaped M3 shape library.

use cursor_core::motion::polar;

pub type P = (f32, f32);

pub const SAMPLES: usize = 256;

#[derive(Clone, Debug, PartialEq)]
pub struct Outline(pub Vec<f32>);

impl Outline {
    pub fn mix(&self, other: &Outline, t: f32) -> Outline {
        Outline(self.0.iter().zip(&other.0).map(|(a, b)| (a + (b - a) * t).max(0.0)).collect())
    }

    pub fn scaled(&self, k: f32) -> Outline {
        Outline(self.0.iter().map(|r| r * k).collect())
    }

    /// Points around `center`, with sample 0 at `rotation` degrees.
    pub fn points(&self, center: P, rotation: f32) -> Vec<P> {
        let step = 360.0 / SAMPLES as f32;
        self.0.iter().enumerate().map(|(i, &r)| polar(center.0, center.1, r, rotation + step * i as f32)).collect()
    }
}

pub fn circle(radius: f32) -> Outline {
    Outline(vec![radius; SAMPLES])
}

/// Samples a closed polygon that is star-shaped about the origin.
fn sample(points: &[P]) -> Outline {
    let n = points.len();
    Outline(
        (0..SAMPLES)
            .map(|i| {
                let (dy, dx) = (i as f32 * 360.0 / SAMPLES as f32).to_radians().sin_cos();
                (0..n)
                    .filter_map(|j| {
                        let (a, b) = (points[j], points[(j + 1) % n]);
                        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                        let det = dx * -ey + dy * ex;
                        if det.abs() < 1e-9 {
                            return None;
                        }
                        let t = (a.0 * -ey + a.1 * ex) / det;
                        let s = (dx * a.1 - dy * a.0) / det;
                        (t >= 0.0 && (-1e-4..=1.0 + 1e-4).contains(&s)).then_some(t)
                    })
                    .fold(0.0, f32::max)
            })
            .collect(),
    )
}

/// Polygon corners rounded with circular arcs of up to `radius` each,
/// clamped so neighbouring arcs never overlap.
fn rounded(corners: &[(P, f32)]) -> Vec<P> {
    let n = corners.len();
    let norm = |(x, y): P| {
        let l = x.hypot(y);
        (x / l, y / l)
    };
    (0..n)
        .flat_map(|i| {
            let (prev, (v, radius), next) = (corners[(i + n - 1) % n].0, corners[i], corners[(i + 1) % n].0);
            let (a, b) = (norm((prev.0 - v.0, prev.1 - v.1)), norm((next.0 - v.0, next.1 - v.1)));
            let half = (a.0 * b.0 + a.1 * b.1).clamp(-1.0, 1.0).acos() / 2.0;
            let reach = 0.5 * (prev.0 - v.0).hypot(prev.1 - v.1).min((next.0 - v.0).hypot(next.1 - v.1));
            let cut = (radius / half.tan()).min(reach);
            let r = cut * half.tan();
            if r < 1e-3 {
                return vec![v];
            }
            let bisector = norm((a.0 + b.0, a.1 + b.1));
            let center = (v.0 + bisector.0 * r / half.sin(), v.1 + bisector.1 * r / half.sin());
            let (t1, t2) = ((v.0 + a.0 * cut, v.1 + a.1 * cut), (v.0 + b.0 * cut, v.1 + b.1 * cut));
            let start = (t1.1 - center.1).atan2(t1.0 - center.0);
            let mut sweep = (t2.1 - center.1).atan2(t2.0 - center.0) - start;
            if sweep > std::f32::consts::PI {
                sweep -= std::f32::consts::TAU;
            } else if sweep < -std::f32::consts::PI {
                sweep += std::f32::consts::TAU;
            }
            (0..=8)
                .map(|k| {
                    let angle = start + sweep * k as f32 / 8.0;
                    (center.0 + r * angle.cos(), center.1 + r * angle.sin())
                })
                .collect()
        })
        .collect()
}

/// Rounded polygon from explicit corners, sampled about the origin.
pub fn custom(corners: &[(P, f32)]) -> Outline {
    sample(&rounded(corners))
}

/// Star with `points` tips alternating between `radius` and `inner * radius`,
/// tips rounded by `outer_round` and valleys by `inner_round` (both in units).
pub fn star(points: usize, radius: f32, inner: f32, outer_round: f32, inner_round: f32, rotation: f32) -> Outline {
    let corners: Vec<(P, f32)> = (0..points * 2)
        .map(|i| {
            let tip = i % 2 == 0;
            let angle = rotation - 90.0 + 180.0 * i as f32 / points as f32;
            let r = if tip { radius } else { radius * inner };
            (polar(0.0, 0.0, r, angle), if tip { outer_round } else { inner_round })
        })
        .collect();
    custom(&corners)
}

pub fn polygon(sides: usize, radius: f32, round: f32, rotation: f32) -> Outline {
    let corners: Vec<(P, f32)> = (0..sides)
        .map(|i| (polar(0.0, 0.0, radius, rotation - 90.0 + 360.0 * i as f32 / sides as f32), round))
        .collect();
    custom(&corners)
}

/// Stadium `length` long and `width` wide along `angle`.
pub fn pill(length: f32, width: f32, angle: f32) -> Outline {
    let half = (length - width).max(0.0) / 2.0;
    let r = width / 2.0;
    let mut points: Vec<P> = (0..=24).map(|k| polar(half, 0.0, r, -90.0 + 180.0 * k as f32 / 24.0)).collect();
    points.extend((0..=24).map(|k| polar(-half, 0.0, r, 90.0 + 180.0 * k as f32 / 24.0)));
    let (s, c) = angle.to_radians().sin_cos();
    sample(&points.into_iter().map(|(x, y)| (x * c - y * s, x * s + y * c)).collect::<Vec<_>>())
}

/// Stretches an outline by `(sx, sy)` about its center.
pub fn stretch(outline: &Outline, sx: f32, sy: f32) -> Outline {
    sample(&outline.points((0.0, 0.0), 0.0).into_iter().map(|(x, y)| (x * sx, y * sy)).collect::<Vec<_>>())
}

// The Material 3 Expressive shape library, at radius `r`.

pub fn cookie(sides: usize, r: f32) -> Outline {
    star(sides, r, 0.8, r, r, 0.0)
}

/// Round leaves around a small hub, reaching radius `r`.
pub fn clover(leaves: usize, r: f32) -> Outline {
    let (offset, leaf) = (0.5 * r, 0.5 * r);
    let discs: Vec<(P, f32)> = (0..leaves)
        .map(|i| (polar(0.0, 0.0, offset, -90.0 + 360.0 * i as f32 / leaves as f32), leaf))
        .chain([((0.0, 0.0), 0.62 * r)])
        .collect();
    Outline(
        (0..SAMPLES)
            .map(|i| {
                let (dy, dx) = (i as f32 * 360.0 / SAMPLES as f32).to_radians().sin_cos();
                discs
                    .iter()
                    .filter_map(|&((cx, cy), radius)| {
                        let along = dx * cx + dy * cy;
                        let gap = radius * radius - (cx * cx + cy * cy) + along * along;
                        (gap >= 0.0).then(|| along + gap.sqrt())
                    })
                    .fold(0.0, f32::max)
            })
            .collect(),
    )
}

pub fn sunny(r: f32) -> Outline {
    star(8, r, 0.8, 0.15 * r, 0.15 * r, 0.0)
}

pub fn soft_burst(r: f32) -> Outline {
    star(10, r, 0.72, 0.12 * r, 0.12 * r, 0.0)
}

pub fn flower(r: f32) -> Outline {
    star(8, r, 0.68, r, 0.08 * r, 0.0)
}

pub fn puffy(r: f32) -> Outline {
    stretch(&star(10, r, 0.82, r, 0.05 * r, 0.0), 1.1, 0.9)
}

pub fn pentagon(r: f32) -> Outline {
    polygon(5, r, 0.3 * r, 0.0)
}

pub fn oval(r: f32) -> Outline {
    stretch(&circle(r), 1.0, 0.7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_samples_are_its_radius() {
        let outline =
            custom(&(0..64).map(|i| (polar(0.0, 0.0, 5.0, i as f32 * 360.0 / 64.0), 0.0)).collect::<Vec<_>>());
        assert!(outline.0.iter().all(|&r| (r - 5.0).abs() < 0.05));
    }

    #[test]
    fn stars_reach_their_tips_and_valleys() {
        let outline = star(4, 10.0, 0.5, 0.0, 0.0, 0.0);
        let (max, min) = outline.0.iter().fold((0.0f32, f32::MAX), |(a, b), &r| (a.max(r), b.min(r)));
        assert!((max - 10.0).abs() < 0.1 && min > 4.0 && min < 5.5, "{max} {min}");
    }

    #[test]
    fn pill_spans_its_length() {
        let outline = pill(20.0, 4.0, 0.0);
        assert!((outline.0[0] - 10.0).abs() < 0.05);
        assert!((outline.0[SAMPLES / 4] - 2.0).abs() < 0.05);
    }
}
