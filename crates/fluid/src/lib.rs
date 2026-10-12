//! Liquid metaball cursors: every state is a few drops of colored liquid
//! melting into one another, and every transition flows, splits and merges.

pub mod cursors;
pub mod field;
pub mod idle;
pub mod render;
pub mod scene;

use cursor_core::{Frame, Palette, Shape, Theme};
use tiny_skia::{Pixmap, Transform};

use crate::scene::{Scene, blend};

pub struct Fluid {
    colors: Palette,
}

impl Theme for Fluid {
    type Scene = Scene;

    const ID: &'static str = "animated_fluid_cursors";
    const TITLE: &'static str = "Animated Fluid";
    const ROLES: &'static [&'static str] = &[
        "arrow",
        "link",
        "text",
        "resize",
        "grab",
        "crosshair",
        "zoom",
        "busy",
        "orbit0",
        "orbit1",
        "orbit2",
        "copy",
        "refuse",
        "border",
        "shadow",
    ];
    const PALETTES: &'static [(&'static str, &'static str)] = &[
        ("carburetor", include_str!("../colors/carburetor.conf")),
        ("catppuccin", include_str!("../colors/catppuccin.conf")),
        ("gruvbox", include_str!("../colors/gruvbox.conf")),
    ];

    fn new(colors: Palette) -> Fluid {
        Fluid { colors }
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
