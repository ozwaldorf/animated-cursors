//! Liquid metaball cursors: every state is a few drops of colored liquid
//! melting into one another, and every transition flows, splits and merges.

pub mod cursors;
pub mod field;
pub mod idle;
pub mod render;
pub mod scene;

use cursor_core::{Frame, Shape, Theme};
use tiny_skia::{Pixmap, Transform};

use crate::scene::{Scene, blend};

pub struct Fluid;

impl Theme for Fluid {
    type Scene = Scene;

    const ID: &'static str = "animated_fluid_cursors";
    const TITLE: &'static str = "Animated Fluid";

    fn cursor(&self, shape: Shape) -> Vec<Frame<Scene>> {
        cursors::frames(shape).into_iter().map(|(scene, delay)| Frame { scene, delay }).collect()
    }

    fn transition(&self, from: Shape, to: Shape, t: f32) -> Scene {
        blend(&cursors::scene(from, 0.0), &cursors::scene(to, 0.0), t)
    }

    fn draw(&self, scene: &Scene, pixmap: &mut Pixmap, transform: Transform) {
        render::draw(scene, pixmap, transform);
    }
}
