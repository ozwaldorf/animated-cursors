//! Easing and geometry helpers in design units.

use std::f32::consts::PI;

/// Symmetric ease: `ease(1 - t) == 1 - ease(t)`, so reversed playback of a
/// transition matches the opposite transition.
pub fn ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Maps `t` into a sub-window of [0, 1], clamped.
pub fn window(t: f32, (start, end): (f32, f32)) -> f32 {
    ((t - start) / (end - start)).clamp(0.0, 1.0)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn polar(x: f32, y: f32, radius: f32, degrees: f32) -> (f32, f32) {
    let (sin, cos) = (degrees * PI / 180.0).sin_cos();
    (x + radius * cos, y + radius * sin)
}

/// Spring-like settle with a little overshoot, as in M3 expressive motion.
pub fn spring(t: f32) -> f32 {
    let (c1, c3) = (1.1, 2.1);
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

/// Damped oscillation that starts and ends at rest.
pub fn wobble(u: f32, cycles: f32) -> f32 {
    (std::f32::consts::TAU * cycles * u).sin() * (1.0 - u).powf(1.5)
}

/// Smooth bump over [start, start + width], 0 elsewhere.
pub fn bump(u: f32, start: f32, width: f32) -> f32 {
    let v = ((u - start) / width).clamp(0.0, 1.0);
    (PI * v).sin().powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_is_symmetric() {
        for step in 0..=10 {
            let t = step as f32 / 10.0;
            assert!((ease(1.0 - t) - (1.0 - ease(t))).abs() < 1e-6);
        }
    }
}
