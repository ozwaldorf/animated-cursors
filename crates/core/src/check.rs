//! Consistency checks for a generated theme, shared by every theme's tests.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::palette::Palette;
use crate::shape::PAIRS;
use crate::theme::{
    Alias, Look, PAINTED, SIZES, TRANSITION_DELAY, TRANSITION_FRAMES, Theme, cursor_links, generate, look, looks,
    order, transition_links,
};
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
/// Generates the theme in every bundled palette, or only in `CURSOR_PALETTE`
/// (a name or file) when set, and checks each, with any `CURSOR_ALIASES`. With
/// `CURSOR_OUTPUT` set, checks the theme already written there instead.
pub fn theme<T: Theme>() {
    let aliases = std::env::var("CURSOR_ALIASES").map_or(Ok(Vec::new()), |list| Alias::parse::<T>(&list)).unwrap();
    let load = |choice: &str| T::new(Palette::load::<T>(choice).unwrap());
    let palette = std::env::var("CURSOR_PALETTE");
    if let Ok(output) = std::env::var("CURSOR_OUTPUT") {
        let theme = load(&palette.unwrap_or_else(|_| T::PALETTES[0].0.to_owned()));
        return checked(&theme, Path::new(&output), &aliases);
    }
    let choices = match palette {
        Ok(choice) => vec![choice],
        Err(_) => T::PALETTES.iter().map(|(name, _)| name.to_string()).collect(),
    };
    for choice in choices {
        generated(&load(&choice), &aliases);
    }
}

/// Generates the theme in its default palette with `aliases` and checks it.
pub fn aliased<T: Theme>(aliases: &[Alias]) {
    generated(&T::new(Palette::load::<T>(T::PALETTES[0].0).unwrap()), aliases);
}

/// Generates the theme into a temporary directory and checks it.
fn generated<T: Theme>(theme: &T, aliases: &[Alias]) {
    let name = &theme.palette().name;
    let output = std::env::temp_dir().join(format!("{}-{name}-{}-check-{}", T::ID, aliases.len(), std::process::id()));
    let _ = fs::remove_dir_all(&output);
    generate(theme, &output, &SIZES, aliases).unwrap();
    checked(theme, &output, aliases);
    fs::remove_dir_all(&output).unwrap();
}

/// Checks a theme written to `output`.
fn checked<T: Theme>(theme: &T, output: &Path, aliases: &[Alias]) {
    let cursors = output.join("cursors");
    let mut largest: HashMap<u32, u32> = HashMap::new();

    let looks: HashMap<Look, _> =
        looks::<T>(aliases).into_iter().map(|look| (look, read(&cursors, look.name()))).collect();
    for (look, sizes) in &looks {
        let expected = order(theme, *look).len();
        assert_eq!(sizes.len(), SIZES.len(), "{look:?}");
        for (size, frames) in sizes {
            let context = format!("{look:?} at {size}");
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
        let name = format!("{}-to-{}", a.name(), b.name());
        let transition = read(&cursors, &name);
        for size in SIZES {
            let frames = &transition[&size];
            let context = format!("{name} at {size}");
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
                let cursor = &looks[&look(shape, aliases)][&size][0];
                assert_eq!((endpoint.xhot, endpoint.yhot), (cursor.xhot, cursor.yhot), "{context} endpoint hotspot");
                assert!(endpoint.pixels == cursor.pixels, "{context} endpoint differs from {shape:?}");
            }
        }
    }

    for (size, span) in largest {
        assert!(span + 6 >= size, "largest cursor spans only {span} of {size} px");
    }

    for (name, target) in transition_links(aliases) {
        assert_eq!(fs::read_link(cursors.join(&name)).unwrap(), Path::new(&target), "{name}");
    }
    for (name, target) in cursor_links(aliases) {
        assert_eq!(fs::read_link(cursors.join(name)).unwrap(), Path::new(target), "{name}");
    }
}
