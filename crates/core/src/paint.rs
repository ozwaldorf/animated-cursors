//! Painting utilities shared by theme renderers.

use tiny_skia::{Pixmap, PixmapPaint, Transform};

/// Separable gaussian blur of premultiplied pixels with transparent edges.
pub fn blur(pixmap: &mut Pixmap, sigma: f32) {
    let radius = (sigma * 3.0).ceil() as isize;
    let kernel: Vec<f32> = (-radius..=radius).map(|i| (-((i * i) as f32) / (2.0 * sigma * sigma)).exp()).collect();
    let total: f32 = kernel.iter().sum();
    let (width, height) = (pixmap.width() as isize, pixmap.height() as isize);
    let mut values: Vec<f32> = pixmap.data().iter().map(|&value| value as f32).collect();
    for (dx, dy) in [(1, 0), (0, 1)] {
        let source = values.clone();
        for y in 0..height {
            for x in 0..width {
                for channel in 0..4 {
                    let sum: f32 = kernel
                        .iter()
                        .zip(-radius..=radius)
                        .filter_map(|(weight, offset)| {
                            let (sx, sy) = (x + offset * dx, y + offset * dy);
                            let inside = (0..width).contains(&sx) && (0..height).contains(&sy);
                            inside.then(|| weight * source[((sy * width + sx) * 4 + channel) as usize])
                        })
                        .sum();
                    values[((y * width + x) * 4 + channel) as usize] = sum / total;
                }
            }
        }
    }
    for (target, value) in pixmap.data_mut().iter_mut().zip(values) {
        *target = value.round().clamp(0.0, 255.0) as u8;
    }
}

pub struct Shadow {
    pub opacity: f32,
    /// Downward offset in design units.
    pub offset: f32,
    /// Blur radius (sigma) in design units.
    pub blur: f32,
}

/// Paints a soft drop shadow of whatever `silhouette` draws; call it before
/// drawing the shape itself.
pub fn drop_shadow(
    pixmap: &mut Pixmap,
    shadow: &Shadow,
    transform: Transform,
    silhouette: impl FnOnce(&mut Pixmap, Transform),
) {
    let mut layer = Pixmap::new(pixmap.width(), pixmap.height()).expect("nonzero size");
    silhouette(&mut layer, transform.pre_translate(0.0, shadow.offset));
    blur(&mut layer, shadow.blur * transform.sx);
    let paint = PixmapPaint { opacity: shadow.opacity, ..PixmapPaint::default() };
    pixmap.draw_pixmap(0, 0, layer.as_ref(), &paint, Transform::identity(), None);
}

/// Premultiplied ARGB words, as stored in Xcursor images.
pub fn argb(pixmap: &Pixmap) -> Vec<u32> {
    pixmap.data().chunks_exact(4).map(|p| u32::from_be_bytes([p[3], p[0], p[1], p[2]])).collect()
}
