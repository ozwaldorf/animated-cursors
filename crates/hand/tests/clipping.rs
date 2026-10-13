//! The glove never pushes deep into the props it holds or passes, in any
//! cursor or transition frame.

use cursor_core::shape::PAIRS;
use cursor_core::{Palette, Shape, Theme};
use hand_cursors::Hand;
use hand_cursors::rig::{Scene, model};
use hand_cursors::solid::Model;
use hand_cursors::vector::V;

/// Fingers may rest on a prop, not sink into it.
const TOLERANCE: f32 = 0.4;

/// Depth of the deepest sampled point inside both the glove and a prop.
fn overlap(colors: &Palette, scene: &Scene) -> f32 {
    let all = model(colors, scene);
    let part = |glove: bool| Model {
        solids: all.solids.iter().copied().filter(|s| (s.body < 2) == glove).collect(),
        blends: if glove { vec![] } else { all.blends.clone() },
    };
    let (glove, props) = (part(true), part(false));
    let every = |m: &Model| (0..m.solids.len() as u16).collect::<Vec<_>>();
    let (g, p) = (every(&glove), every(&props));
    let steps = 10;
    let grid = |i: usize| i as f32 / steps as f32 * 2.0 - 1.0;
    props
        .solids
        .iter()
        .flat_map(|solid| {
            let (c, r) = solid.bound();
            (0..=steps).flat_map(move |i| {
                (0..=steps).flat_map(move |j| (0..=steps).map(move |k| c + V(grid(i), grid(j), grid(k)) * r))
            })
        })
        .map(|at| (-glove.field(&g, at).0).min(-props.field(&p, at).0))
        .fold(0.0, f32::max)
}

#[test]
fn glove_does_not_clip_props() {
    let colors = Palette::load::<Hand>("carburetor").unwrap();
    let theme = Hand::new(Palette::load::<Hand>("carburetor").unwrap());
    for shape in Shape::ALL {
        for (i, frame) in theme.cursor(shape).iter().enumerate() {
            let depth = overlap(&colors, &frame.scene);
            assert!(depth < TOLERANCE, "{} frame {i} clips by {depth:.2}", shape.name());
        }
    }
    for (a, b) in PAIRS {
        for i in 1..23 {
            let depth = overlap(&colors, &theme.transition(a, b, i as f32 / 23.0));
            assert!(depth < TOLERANCE, "{}-to-{} frame {i} clips by {depth:.2}", a.name(), b.name());
        }
    }
}
