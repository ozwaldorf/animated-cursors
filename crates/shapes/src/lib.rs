//! Material 3 Expressive shape cursors: every state is a shape from the M3
//! library in its own palette color, and every transition is a springy shape morph.

pub mod cursors;
pub mod outline;
pub mod render;
pub mod scene;

use cursor_core::{Frame, Palette, Shape, Theme};
use tiny_skia::{Pixmap, Transform};

use crate::scene::{Scene, blend};

pub struct Shapes {
    colors: Palette,
}

impl Theme for Shapes {
    type Scene = Scene;

    const ID: &'static str = "animated_shapes_cursors";
    const TITLE: &'static str = "Animated Shapes";
    const ROLES: &'static [&'static str] = &[
        "arrow",
        "link",
        "help",
        "text",
        "resize",
        "grab",
        "grabbing",
        "copy",
        "refuse",
        "crosshair",
        "zoom",
        "load0",
        "load1",
        "load2",
        "load3",
        "load4",
        "load5",
        "load6",
        "glyph",
        "border",
        "shadow",
    ];
    const PALETTES: &'static [(&'static str, &'static str)] = &[
        ("carburetor", include_str!("../colors/carburetor.conf")),
        ("catppuccin", include_str!("../colors/catppuccin.conf")),
        ("gruvbox", include_str!("../colors/gruvbox.conf")),
    ];

    fn new(colors: Palette) -> Shapes {
        Shapes { colors }
    }

    fn palette(&self) -> &Palette {
        &self.colors
    }

    fn cursor(&self, shape: Shape) -> Vec<Frame<Scene>> {
        cursors::frames(&self.colors, shape).into_iter().map(|(scene, delay)| Frame { scene, delay }).collect()
    }

    fn transition(&self, from: Shape, to: Shape, t: f32) -> Scene {
        blend(&cursors::scene(&self.colors, from, 0.0), &cursors::scene(&self.colors, to, 0.0), t)
    }

    fn draw(&self, scene: &Scene, pixmap: &mut Pixmap, transform: Transform) {
        render::draw(&self.colors, scene, pixmap, transform);
    }
}
