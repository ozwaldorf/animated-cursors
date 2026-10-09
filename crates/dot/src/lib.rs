//! Minimal dot and ring cursors built from keyed bars, chevrons and arcs.

pub mod idle;
pub mod render;
pub mod rig;

use cursor_core::{Frame, Shape, Theme};
use tiny_skia::{Pixmap, Transform};

use crate::rig::{Parts, blend, parts};

const SPIN_FRAMES: usize = 36;
const SPIN_DELAY: u32 = 25;

pub struct Dot;

impl Theme for Dot {
    type Scene = Parts;

    const ID: &'static str = "animated_dot_cursors";
    const TITLE: &'static str = "Animated Dot";

    fn hotspot(&self, _: Shape) -> (f32, f32) {
        (16.0, 16.0)
    }

    fn cursor(&self, shape: Shape) -> Vec<Frame<Parts>> {
        if shape.is_busy() {
            let phase = |index: usize| index as f32 / SPIN_FRAMES as f32;
            (0..SPIN_FRAMES).map(|index| Frame { scene: parts(shape, phase(index)), delay: SPIN_DELAY }).collect()
        } else {
            let (hold, count) = idle::timing(shape);
            std::iter::once(Frame { scene: parts(shape, 0.0), delay: hold })
                .chain(
                    (1..count)
                        .map(|i| Frame { scene: idle::gesture(shape, i as f32 / count as f32), delay: idle::STEP }),
                )
                .collect()
        }
    }

    fn transition(&self, from: Shape, to: Shape, t: f32) -> Parts {
        blend(&parts(from, 0.0), &parts(to, 0.0), t)
    }

    fn draw(&self, scene: &Parts, pixmap: &mut Pixmap, transform: Transform) {
        render::draw(scene, pixmap, transform);
    }
}
