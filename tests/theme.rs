use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use dot_cursors::rig::Shape;
use dot_cursors::theme::{self, DELAY, FRAMES, LEGACY_ALIASES, PAIRS, SIZES, SPIN_DELAY, SPIN_FRAMES};
use dot_cursors::xcursor::{Image, decode};

fn read(cursors: &Path, name: &str) -> HashMap<u32, Vec<Image>> {
    let mut sizes: HashMap<u32, Vec<Image>> = HashMap::new();
    for image in decode(&fs::read(cursors.join(name)).unwrap()).expect(name) {
        sizes.entry(image.size).or_default().push(image);
    }
    sizes
}

#[test]
fn generated_theme_is_complete_and_consistent() {
    let output = std::env::temp_dir().join(format!("dot-cursors-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&output);
    theme::generate(&output, None).unwrap();
    let cursors: PathBuf = output.join("cursors");

    let shapes: HashMap<Shape, _> = Shape::ALL.into_iter().map(|shape| (shape, read(&cursors, shape.name()))).collect();
    for (shape, sizes) in &shapes {
        assert_eq!(sizes.len(), SIZES.len(), "{shape:?}");
        for frames in sizes.values() {
            let (count, delay) = if shape.is_spinner() { (SPIN_FRAMES, SPIN_DELAY) } else { (1, 0) };
            assert_eq!(frames.len(), count, "{shape:?}");
            assert!(frames.iter().all(|image| image.delay == delay));
        }
    }

    for (a, b) in PAIRS {
        let names = theme::transition_names(a, b);
        let transition = read(&cursors, &names[0]);
        for size in SIZES {
            let frames = &transition[&size];
            assert_eq!(frames.len(), FRAMES, "{}", names[0]);
            for image in frames {
                assert_eq!((image.width, image.height, image.xhot, image.yhot), (size, size, size / 2, size / 2));
                assert_eq!(image.delay, DELAY);
                assert!(image.pixels.iter().any(|pixel| pixel >> 24 > 0));
            }
            assert_eq!(frames[0].pixels, shapes[&a][&size][0].pixels, "{} start at {size}", names[0]);
            assert_eq!(frames[FRAMES - 1].pixels, shapes[&b][&size][0].pixels, "{} end at {size}", names[0]);
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
