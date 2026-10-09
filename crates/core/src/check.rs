//! Consistency checks for a generated theme, shared by every theme's tests.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::shape::{LEGACY_ALIASES, PAIRS, Shape};
use crate::theme::{GRID, SIZES, TRANSITION_DELAY, TRANSITION_FRAMES, Theme, generate, transition_names};
use crate::xcursor::{Image, decode};

fn read(cursors: &Path, name: &str) -> HashMap<u32, Vec<Image>> {
    let mut sizes: HashMap<u32, Vec<Image>> = HashMap::new();
    for image in decode(&fs::read(cursors.join(name)).unwrap()).expect(name) {
        sizes.entry(image.size).or_default().push(image);
    }
    sizes
}

/// Asserts that a transition endpoint, drawn on its larger canvas, matches the
/// ordinary cursor pixel for pixel and that nothing falls outside the cursor.
fn assert_endpoint(endpoint: &Image, cursor: &Image, context: &str) {
    let (dx, dy) = (endpoint.xhot as i64 - cursor.xhot as i64, endpoint.yhot as i64 - cursor.yhot as i64);
    for (index, &pixel) in endpoint.pixels.iter().enumerate() {
        let (x, y) = ((index as u32 % endpoint.width) as i64 - dx, (index as u32 / endpoint.width) as i64 - dy);
        let inside = (0..cursor.width as i64).contains(&x) && (0..cursor.height as i64).contains(&y);
        let expected = if inside { cursor.pixels[(y * cursor.width as i64 + x) as usize] } else { 0 };
        assert_eq!(pixel, expected, "{context} at ({x}, {y})");
    }
}

/// Asserts that nothing but a shadow tail touches the canvas edge; shadows
/// peak near alpha 90, borders and fills at 255.
fn assert_unclipped(image: &Image, context: &str) {
    let (width, height) = (image.width as usize, image.height as usize);
    for (index, pixel) in image.pixels.iter().enumerate() {
        let (x, y) = (index % width, index / width);
        let edge = x == 0 || y == 0 || x == width - 1 || y == height - 1;
        assert!(!edge || pixel >> 24 <= 48, "{context} is clipped at ({x}, {y})");
    }
}

/// Generates the theme into a temporary directory and checks it.
pub fn theme<T: Theme>(theme: &T) {
    let output = std::env::temp_dir().join(format!("{}-check-{}", T::ID, std::process::id()));
    let _ = fs::remove_dir_all(&output);
    generate(theme, &output, None).unwrap();
    let cursors = output.join("cursors");

    let shapes: HashMap<Shape, _> = Shape::ALL.into_iter().map(|shape| (shape, read(&cursors, shape.name()))).collect();
    for (shape, sizes) in &shapes {
        let expected = theme.cursor(*shape).len();
        assert_eq!(sizes.len(), SIZES.len(), "{shape:?}");
        for (size, frames) in sizes {
            let (x, y) = theme.hotspot(*shape);
            assert_eq!((x * *size as f32 / GRID).fract(), 0.0, "{shape:?} hotspot at {size}");
            assert_eq!((y * *size as f32 / GRID).fract(), 0.0, "{shape:?} hotspot at {size}");
            assert_eq!(frames.len(), expected, "{shape:?}");
            assert!(frames.iter().all(|image| image.width == *size && image.height == *size));
            frames.iter().for_each(|image| assert_unclipped(image, &format!("{shape:?} at {size}")));
            assert!(expected == 1 || frames.iter().all(|image| image.delay > 0), "{shape:?} delays");
        }
    }

    for (a, b) in PAIRS {
        let names = transition_names(a, b);
        let transition = read(&cursors, &names[0]);
        for size in SIZES {
            let frames = &transition[&size];
            assert_eq!(frames.len(), TRANSITION_FRAMES, "{}", names[0]);
            for image in frames {
                assert_eq!((image.width, image.height, image.xhot, image.yhot), (2 * size, 2 * size, size, size));
                assert_eq!(image.delay, TRANSITION_DELAY);
                assert!(image.pixels.iter().any(|pixel| pixel >> 24 > 0), "{} is empty", names[0]);
                assert_unclipped(image, &format!("{} at {size}", names[0]));
            }
            assert_endpoint(&frames[0], &shapes[&a][&size][0], &format!("{} start at {size}", names[0]));
            let end = &frames[TRANSITION_FRAMES - 1];
            assert_endpoint(end, &shapes[&b][&size][0], &format!("{} end at {size}", names[0]));
        }
        for name in &names[1..] {
            let link = cursors.join(name);
            assert!(link.is_symlink(), "{name}");
            assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(cursors.join(&names[0])).unwrap());
        }
    }

    let aliases = Shape::ALL.into_iter().flat_map(|shape| shape.aliases().iter().map(move |name| (shape, *name)));
    let legacy = LEGACY_ALIASES.into_iter().flat_map(|(shape, names)| names.iter().map(move |name| (shape, *name)));
    for (shape, name) in aliases.chain(legacy) {
        assert_eq!(fs::read_link(cursors.join(name)).unwrap(), Path::new(shape.name()), "{name}");
    }
    fs::remove_dir_all(&output).unwrap();
}
