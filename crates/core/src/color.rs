//! Linear-light sRGB colors and perceptual blending through OKLab.

use crate::motion::lerp;

/// Linear-light sRGB color.
pub type Color = [f32; 3];

pub fn hex(rgb: u32) -> Color {
    let channel = |shift: u32| {
        let c = ((rgb >> shift) & 0xff) as f32 / 255.0;
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    [channel(16), channel(8), channel(0)]
}

/// Gamma-encodes a linear channel to sRGB in [0, 1].
pub fn encode(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
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
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let (a, b) = (oklab(a), oklab(b));
    linear(std::array::from_fn(|i| lerp(a[i], b[i], t)))
}
