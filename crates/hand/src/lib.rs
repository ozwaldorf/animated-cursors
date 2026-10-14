//! Cel-shaded 3D glove cursors after Master Hand: a ray-marched glove in flat
//! tones and ink lines that points, grabs and holds props, and turns and
//! flexes between them.

pub mod cursors;
pub mod idle;
pub mod render;
pub mod rig;
pub mod solid;
pub mod vector;

use cursor_core::{Frame, Palette, Shape, Theme};
use tiny_skia::{Pixmap, Transform};

use crate::rig::{Scene, blend};

pub struct Hand {
    colors: Palette,
}

impl Theme for Hand {
    type Scene = Scene;

    const ID: &'static str = "animated_hand_cursors";
    const TITLE: &'static str = "Animated Hand";
    const ROLES: &'static [&'static str] =
        &["glove", "shade", "ink", "text", "resize", "crosshair", "zoom", "help", "busy", "copy", "refuse", "shadow"];
    const PALETTES: &'static [(&'static str, &'static str)] = &[
        ("carburetor", include_str!("../colors/carburetor.conf")),
        ("catppuccin", include_str!("../colors/catppuccin.conf")),
        ("gruvbox", include_str!("../colors/gruvbox.conf")),
    ];

    fn new(colors: Palette) -> Hand {
        Hand { colors }
    }

    fn palette(&self) -> &Palette {
        &self.colors
    }

    fn cursor(&self, shape: Shape) -> Vec<Frame<Scene>> {
        cursors::frames(shape).0.into_iter().map(|(scene, delay)| Frame { scene, delay }).collect()
    }

    fn order(&self, shape: Shape) -> Vec<usize> {
        cursors::frames(shape).1
    }

    fn transition(&self, from: Shape, to: Shape, t: f32) -> Scene {
        blend(&cursors::scene(from, 0.0), &cursors::scene(to, 0.0), t)
    }

    fn draw(&self, scene: &Scene, pixmap: &mut Pixmap, transform: Transform) {
        render::draw(&self.colors, scene, pixmap, transform);
    }
}
