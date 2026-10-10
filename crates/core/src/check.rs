//! Consistency checks for a generated theme, shared by every theme's tests.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::shape::{LEGACY_ALIASES, PAIRS, Shape};
use crate::theme::{PAINTED, SIZES, TRANSITION_DELAY, TRANSITION_FRAMES, Theme, generate, transition_names};
use crate::xcursor::{Image, decode};

fn read(cursors: &Path, name: &str) -> HashMap<u32, Vec<Image>> {
    let mut sizes: HashMap<u32, Vec<Image>> = HashMap::new();
    for image in decode(&fs::read(cursors.join(name)).unwrap()).expect(name) {
        sizes.entry(image.size).or_default().push(image);
    }
    sizes
}

/// Asserts the image is exactly its nominal size with the hotspot inside.
fn assert_shape(image: &Image, context: &str) {
    assert_eq!((image.width, image.height), (image.size, image.size), "{context} size");
    assert!(image.xhot < image.width && image.yhot < image.height, "{context} hotspot");
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

/// Longest side of the painted area, in pixels.
fn span(image: &Image) -> u32 {
    let painted = image.pixels.iter().enumerate().filter(|(_, pixel)| *pixel >> 24 > PAINTED as u32);
    let (x0, y0, x1, y1) = painted.fold((u32::MAX, u32::MAX, 0, 0), |(x0, y0, x1, y1), (index, _)| {
        let (x, y) = (index as u32 % image.width, index as u32 / image.width);
        (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1))
    });
    x1.saturating_sub(x0).max(y1.saturating_sub(y0))
}

/// Generates the theme into a temporary directory and checks it.
pub fn theme<T: Theme>(theme: &T) {
    let output = std::env::temp_dir().join(format!("{}-check-{}", T::ID, std::process::id()));
    let _ = fs::remove_dir_all(&output);
    generate(theme, &output, None, &SIZES).unwrap();
    let cursors = output.join("cursors");
    let mut largest: HashMap<u32, u32> = HashMap::new();

    let shapes: HashMap<Shape, _> = Shape::ALL.into_iter().map(|shape| (shape, read(&cursors, shape.name()))).collect();
    for (shape, sizes) in &shapes {
        let expected = theme.cursor(*shape).len();
        assert_eq!(sizes.len(), SIZES.len(), "{shape:?}");
        for (size, frames) in sizes {
            let context = format!("{shape:?} at {size}");
            assert_eq!(frames.len(), expected, "{context}");
            for image in frames {
                assert_shape(image, &context);
                assert_eq!((image.xhot, image.yhot), (frames[0].xhot, frames[0].yhot), "{context} hotspot moves");
                assert_unclipped(image, &context);
                let span = span(image);
                largest.entry(*size).and_modify(|max| *max = span.max(*max)).or_insert(span);
            }
            assert!(expected == 1 || frames.iter().all(|image| image.delay > 0), "{context} delays");
        }
    }

    for (a, b) in PAIRS {
        let names = transition_names(a, b);
        let transition = read(&cursors, &names[0]);
        for size in SIZES {
            let frames = &transition[&size];
            let context = format!("{} at {size}", names[0]);
            assert_eq!(frames.len(), TRANSITION_FRAMES, "{context}");
            for image in frames {
                assert_shape(image, &context);
                assert_eq!(image.delay, TRANSITION_DELAY);
                assert!(image.pixels.iter().any(|pixel| pixel >> 24 > 0), "{context} is empty");
                assert_unclipped(image, &context);
                let span = span(image);
                largest.entry(size).and_modify(|max| *max = span.max(*max)).or_insert(span);
            }
            for (endpoint, shape) in [(&frames[0], a), (&frames[TRANSITION_FRAMES - 1], b)] {
                let cursor = &shapes[&shape][&size][0];
                assert_eq!((endpoint.xhot, endpoint.yhot), (cursor.xhot, cursor.yhot), "{context} endpoint hotspot");
                assert!(endpoint.pixels == cursor.pixels, "{context} endpoint differs from {shape:?}");
            }
        }
        for name in &names[1..] {
            let link = cursors.join(name);
            assert!(link.is_symlink(), "{name}");
            assert_eq!(fs::canonicalize(&link).unwrap(), fs::canonicalize(cursors.join(&names[0])).unwrap());
        }
    }

    for (size, span) in largest {
        assert!(span + 6 >= size, "largest cursor spans only {span} of {size} px");
    }

    let aliases = Shape::ALL.into_iter().flat_map(|shape| shape.aliases().iter().map(move |name| (shape, *name)));
    let legacy = LEGACY_ALIASES.into_iter().flat_map(|(shape, names)| names.iter().map(move |name| (shape, *name)));
    for (shape, name) in aliases.chain(legacy) {
        assert_eq!(fs::read_link(cursors.join(name)).unwrap(), Path::new(shape.name()), "{name}");
    }
    fs::remove_dir_all(&output).unwrap();
}
