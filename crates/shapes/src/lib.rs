//! Material 3 Expressive shape cursors: every state is a shape from the M3
//! library in its own color, and every transition is a springy shape morph.

pub mod cursors;
pub mod outline;
pub mod render;
pub mod scene;

use cursor_core::{Frame, Shape, Theme};
use tiny_skia::{Pixmap, Transform};

use crate::scene::{Scene, blend};

pub struct Shapes;

impl Theme for Shapes {
    type Scene = Scene;

    const ID: &'static str = "animated_shapes_cursors";
    const TITLE: &'static str = "Animated Shapes";

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
