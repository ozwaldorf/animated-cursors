//! Cel-shaded 3D glove cursors after Master Hand: a ray-marched glove in flat
//! tones and ink lines that points, grabs and holds props, and turns and
//! flexes between them.

pub mod cursors;
pub mod idle;
pub mod render;
pub mod rig;
pub mod solid;
pub mod vector;

use cursor_core::{Frame, Look, Palette, Shape, Theme};
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
    const GESTURES: &'static [&'static str] = &cursors::GESTURES;

    fn new(colors: Palette) -> Hand {
        Hand { colors }
    }

    fn palette(&self) -> &Palette {
        &self.colors
    }

    fn cursor(&self, shape: Shape) -> Vec<Frame<Scene>> {
        frames(Look::Shape(shape))
    }

    fn order(&self, shape: Shape) -> Vec<usize> {
        cursors::frames(Look::Shape(shape)).1
    }

    fn gesture(&self, name: &'static str) -> Vec<Frame<Scene>> {
        frames(Look::Gesture(name))
    }

    fn gesture_order(&self, name: &'static str) -> Vec<usize> {
        cursors::frames(Look::Gesture(name)).1
    }

    fn transition(&self, from: Shape, to: Shape, t: f32) -> Scene {
        self.morph(Look::Shape(from), Look::Shape(to), t)
    }

    fn morph(&self, from: Look, to: Look, t: f32) -> Scene {
        blend(&cursors::rest(from), &cursors::rest(to), t)
    }

    fn draw(&self, scene: &Scene, pixmap: &mut Pixmap, transform: Transform) {
        render::draw(&self.colors, scene, pixmap, transform, true);
    }

    fn silhouette(&self, scene: &Scene, pixmap: &mut Pixmap, transform: Transform) {
        render::draw(&self.colors, scene, pixmap, transform, false);
    }
}

fn frames(look: Look) -> Vec<Frame<Scene>> {
    cursors::frames(look).0.into_iter().map(|(scene, delay)| Frame { scene, delay }).collect()
}
