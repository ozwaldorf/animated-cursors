//! Each cursor as a posed glove with its props, in design units with the
//! hotspot at the origin.

use std::f32::consts::{PI, TAU};

use cursor_core::Shape;

use crate::idle;
use crate::rig::{
    GRIP, HOLD_TIP, HOLD_TURN, Hold, Kind, Pose, Prop, Props, Scene, hips, index_tip, leg_length, palm_facing,
    pinch_tip, prop, reach, reach_root,
};
use crate::vector::V;

const ORIGIN: V = V(0.0, 0.0, 0.0);
/// Middle of the hand's silhouette, in its own frame.
const MIDDLE: V = V(0.0, -1.6, 0.0);
/// Middle of the fingers held straight, in the hand's frame.
pub const FINGERS: V = V(0.0, -3.4, 0.0);

/// Index finger out, the others curled, back of the hand to the viewer.
fn point() -> Pose {
    Pose {
        at: ORIGIN,
        size: 1.0,
        turn: -25.0,
        tip: 15.0,
        roll: 0.0,
        spread: [-4.0, 2.0, 6.0, 10.0],
        curl: [0.0, 0.92, 0.95, 0.97],
        flex: [[0.0; 3]; 4],
        thumb: [25.0, 55.0, 0.75],
    }
}

/// Palm to the viewer, fingers spread.
fn open() -> Pose {
    Pose {
        turn: 180.0,
        tip: 0.0,
        spread: [-14.0, -3.0, 8.0, 20.0],
        curl: [0.08, 0.06, 0.08, 0.1],
        thumb: [62.0, 10.0, 0.1],
        ..point()
    }
}

/// Closed fist, palm to the viewer.
fn fist() -> Pose {
    Pose { turn: 215.0, tip: 0.0, spread: [-3.0, 0.0, 3.0, 6.0], curl: [1.0; 4], thumb: [35.0, 20.0, 0.9], ..point() }
}

/// `pose` rolled and sized with its index fingertip on `target`.
fn aimed(pose: Pose, roll: f32, size: f32, target: V) -> Pose {
    let pose = Pose { roll, size, ..pose };
    pose.pinned(index_tip(&pose), target)
}

/// `pose` rolled and sized with the middle of the hand on `target`.
fn centered(pose: Pose, roll: f32, size: f32, target: V) -> Pose {
    Pose { roll, size, ..pose }.pinned(MIDDLE, target)
}

/// `pose` palm down with the fingers up, back of the hand to the viewer,
/// centered.
fn palm_down(pose: Pose) -> Pose {
    centered(Pose { turn: pose.turn - 180.0, tip: 10.0, ..pose }, 0.0, 1.0, ORIGIN)
}

/// The pointing hand turned to show its thumb, the index curled down a little
/// at rest so its tip can lift to tap, bent joint by joint as the poke bends
/// it further.
fn pointing() -> Pose {
    let mut flex = [[0.0; 3]; 4];
    flex[0] = [10.0, 24.0, 16.8];
    Pose { turn: 30.0, flex, ..point() }
}

/// Open hand at rest, fingers splayed and gently curled, more toward the
/// pinky, with the thumb resting out to the side.
fn relaxed() -> Pose {
    Pose {
        turn: 40.0,
        spread: [-20.0, -5.0, 10.0, 25.0],
        curl: [0.2, 0.22, 0.3, 0.38],
        thumb: [48.0, 22.0, 0.25],
        ..point()
    }
}

/// The relaxed hand leaning up and left, the index fingertip on the hotspot.
fn arrow() -> Pose {
    aimed(relaxed(), -24.0, 1.0, ORIGIN)
}

/// Fingers and thumb closed around a bar along the hand's x axis, the palm
/// tipped back so the cuff comes out toward the viewer. The loop they close
/// leaves `GRIP` 1.4 units clear.
fn hold() -> Pose {
    Pose {
        turn: HOLD_TURN,
        tip: HOLD_TIP,
        spread: [-2.0, 0.0, 2.0, 4.0],
        curl: [0.45, 0.47, 0.5, 0.55],
        thumb: [20.0, 70.0, 0.9],
        ..point()
    }
}

/// Roll of a hand holding a bar along `angle`, either way round, from the
/// side that keeps the thumb up.
fn upright(angle: f32) -> f32 {
    -(-angle).rem_euclid(180.0)
}

/// `hold` of `size` closed on a bar through `at` along `angle`.
fn holding(at: V, angle: f32, size: f32) -> Pose {
    Pose { roll: upright(angle), size, ..hold() }.pinned(GRIP, at)
}

/// Index and middle held together as a barrel and the thumb cocked up as a
/// hammer, the others curled into a grip: a hand held out in front, seen
/// from behind as the barrel points away and to the left.
fn gun() -> Pose {
    Pose {
        turn: 180.0,
        tip: 45.0,
        spread: [-1.0, 1.0, 6.0, 10.0],
        curl: [0.0, 0.0, 0.95, 0.97],
        thumb: [80.0, 0.0, 0.0],
        ..point()
    }
}

/// The index raised and bent slightly, the others curled, back of the hand
/// to the viewer, its knuckle on the hotspot so the finger wags about it.
fn wagging() -> Pose {
    let mut flex = [[0.0; 3]; 4];
    flex[0] = [6.0, 14.0, 9.8];
    Pose { turn: -10.0, tip: 5.0, flex, ..point() }.pinned(reach_root(0), ORIGIN)
}

/// Point at `radius` and `degrees` from the hotspot.
fn around(radius: f32, degrees: f32) -> V {
    let (s, c) = degrees.to_radians().sin_cos();
    V(c * radius, s * radius, 0.0)
}

/// `prop` held by a hand of `size` at `grip`, a point in its own frame, along
/// `angle`. The prop rides in the hand's frame, so it stays in the grip and
/// turns with the hand.
fn grasped(prop: Prop, grip: V, angle: f32, size: f32) -> Scene {
    let (s, c) = prop.roll.to_radians().sin_cos();
    let at = prop.at + V(c * grip.0 - s * grip.1, s * grip.0 + c * grip.1, grip.2) * prop.size;
    let hand = holding(at, angle, size);
    let held = prop.held(Hold { offset: grip, twist: prop.roll - hand.roll });
    Scene { hand, props: vec![("prop", held)] }
}

/// Fingers held together and bent at the knuckles to meet the thumb tip, like
/// a beak, thumb side to the viewer, its tip on the hotspot and pointing
/// across `axis`, the line it drags along in degrees on screen. Of the two
/// ways across, it takes the one that rolls the hand least from upright.
fn pinch(axis: f32) -> Pose {
    let beak = Pose {
        turn: 90.0,
        tip: 0.0,
        spread: [-2.0, 0.0, 2.0, 4.0],
        curl: [0.0; 4],
        flex: [[96.0, 21.0, 14.7]; 4],
        thumb: [30.0, 60.0, 0.4],
        ..point()
    };
    let along = palm_facing(&beak);
    let across = (axis + 90.0 - along.1.atan2(along.0).to_degrees()).rem_euclid(180.0);
    // A tie rolls left, as default and pointer lean.
    let roll = if across > 89.9 { across - 180.0 } else { across };
    Pose { roll, ..beak }.pinned(pinch_tip(&beak), ORIGIN)
}

/// The hand itself as the caret: flat and edge on, fingers together, the
/// thumb side to the viewer with the thumb eased off the hand, rolled by
/// `roll` and centered on its fingers.
fn blade(roll: f32) -> Pose {
    let flat = Pose {
        turn: 90.0,
        tip: 0.0,
        spread: [-2.0, 0.0, 2.0, 4.0],
        curl: [0.0; 4],
        thumb: [35.0, 10.0, 0.15],
        ..point()
    };
    Pose { roll, size: 0.86, ..flat }.pinned(FINGERS, ORIGIN)
}

/// A ring of `radius` held at its rim at `degrees` from the center, the grip
/// a little inside it so the curve leaves the straight row of fingers evenly.
fn rimmed(kind: Kind, radius: f32, degrees: f32, size: f32) -> Scene {
    grasped(prop(kind, ORIGIN, 0.0), around(radius - 0.3, degrees), degrees + 90.0, size)
}

fn badged(kind: Kind) -> Scene {
    let badge = prop(kind, V(6.6, 6.5, 4.5), 0.0);
    Scene { hand: palm_down(fist()), props: vec![("badge", badge.sized(0.8))] }
}

/// The magnifier held by its handle, from above.
fn lens(size: f32, plus: f32) -> Scene {
    grasped(prop(Kind::Lens { plus }, ORIGIN, 0.0).sized(size), around(8.4, 45.0), 225.0, 0.8)
}

/// Walking on the index and middle fingers at `phase` through a stride, seen
/// from the side with the palm raised behind them like a torso. Each leg
/// plants and slides back as on a treadmill, then lifts and swings forward.
/// The body rides as high as the planted legs allow, vaulting over a nearly
/// straight leg and dipping as the feet change; it rocks and twists with
/// the stride, and the thumb swings against the legs like an arm.
fn wait(phase: f32) -> Scene {
    // Where a foot lands ahead of the hips and leaves behind them.
    const FRONT: f32 = 2.4;
    const BACK: f32 = 2.0;
    const LIFT: f32 = 1.5;
    /// Ground below the resting hips.
    const HEIGHT: f32 = 7.6;
    /// How far a planted leg straightens, as a share of its length.
    const STAND: f32 = 0.98;
    // Heading left, the index leg nearest.
    const AHEAD: f32 = -1.0;
    let walker = Pose {
        turn: 90.0,
        tip: 0.0,
        spread: [-6.0, 4.0, 8.0, 11.0],
        curl: [0.0, 0.0, 0.78, 0.85],
        thumb: [25.0 + 15.0 * (TAU * phase).sin(), 40.0, 0.5],
        ..point()
    };
    // Fingers down and forward, the palm tilted up behind them like a torso.
    let rest = centered(walker, -165.0, 1.0, ORIGIN);
    let (hip, ground) = (hips(&rest).0, hips(&rest).1 + HEIGHT);
    // The body rocks and twists toward the leg it stands on, pivoting at the
    // hips.
    let sway = (TAU * phase).sin();
    let swayed = Pose { roll: rest.roll - 5.0 * sway, turn: rest.turn - 10.0 * sway, ..rest };
    let level = swayed.pinned((reach_root(0) + reach_root(1)) * 0.5, hips(&rest));
    // A planted foot slides back steadily; a lifted one gets forward early,
    // then comes down onto a leg reaching out in front.
    let at = |reach: f32| hip + AHEAD * reach;
    let feet = [phase, phase + 0.5].map(|p| match p.rem_euclid(1.0) {
        p if p < 0.5 => (at(FRONT - (FRONT + BACK) * 2.0 * p), ground),
        p => {
            let s = 2.0 * p - 1.0;
            let out = 1.0 - (1.0 - s).powi(2);
            (at(-BACK + (FRONT + BACK) * out), ground - LIFT * (PI * s).sin())
        }
    });
    // Lower the body just enough that each foot is within its leg's reach,
    // measuring each leg as long as the shorter so the body rides evenly.
    let depth = |leg: usize| leg_length(leg) - reach_root(leg).1;
    let even = |leg: usize| leg_length(leg) - (depth(leg) - depth(0).min(depth(1)));
    let drop = (0..2)
        .map(|leg| {
            let knuckle = level.place(reach_root(leg));
            let length = STAND * even(leg);
            let (dx, y) = (feet[leg].0 - knuckle.0, feet[leg].1);
            y - (length * length - dx * dx).max(0.0).sqrt() - knuckle.1
        })
        .fold(f32::NEG_INFINITY, f32::max);
    let pose = Pose { at: level.at + V(0.0, drop, 0.0), ..level };
    let mut flex = pose.flex;
    for leg in 0..2 {
        let depth = pose.place(reach_root(leg)).2;
        flex[leg] = reach(&pose, leg, V(feet[leg].0, feet[leg].1, depth));
    }
    Scene { hand: Pose { flex, ..pose }, props: Props::new() }
}

/// The gyro spinning and bobbing on the hotspot, over the relaxed hand turned
/// palm up beneath it, swaying gently.
fn progress(phase: f32) -> Scene {
    // Turned the way a right hand rolls over to show its palm.
    let (s, c) = (TAU * phase).sin_cos();
    let palm_up = Pose { turn: 250.0 + 8.0 * c, roll: -80.0 + 5.0 * s, ..relaxed() };
    let hand = palm_up.pinned(V(0.0, -2.0, -6.8), ORIGIN);
    let hover = V(0.0, 0.4 * (2.0 * TAU * phase).sin(), 0.0);
    let gyro = Prop { spin: 360.0 * phase, ..prop(Kind::Gyro, hover, 0.0).sized(0.9) };
    Scene { hand, props: vec![("busy", gyro)] }
}

/// A question mark floating over the relaxed hand turned palm up beneath it,
/// as in progress.
fn help() -> Scene {
    let palm_up = Pose { turn: 250.0, roll: -80.0, ..relaxed() };
    let hand = palm_up.pinned(V(0.0, -2.0, -6.8), ORIGIN);
    Scene { hand, props: vec![("help", prop(Kind::Query, V(0.0, -2.0, 0.0), 0.0).sized(0.8))] }
}

/// The scene of a cursor; `phase` in [0, 1) drives busy loops.
pub fn scene(shape: Shape, phase: f32) -> Scene {
    let bare = |hand: Pose| Scene { hand, props: Props::new() };
    match shape {
        Shape::Default => bare(arrow()),
        Shape::Pointer => bare(aimed(pointing(), -6.0, 1.0, ORIGIN)),
        Shape::Progress => progress(phase),
        Shape::Wait => wait(phase),
        Shape::Text => bare(blade(0.0)),
        // Palm down, fingers pointing left.
        Shape::VerticalText => bare(blade(-90.0)),
        Shape::EwResize | Shape::ColResize => bare(pinch(0.0)),
        Shape::NsResize => bare(pinch(90.0)),
        Shape::NeswResize => bare(pinch(-45.0)),
        Shape::NwseResize => bare(pinch(45.0)),
        // The open hand hangs a little below the hotspot, so closing on it
        // reaches forward.
        Shape::Grab => bare(palm_down(open()).pinned(V(0.0, -4.0, 0.0), ORIGIN)),
        Shape::Grabbing => bare(palm_down(fist())),
        Shape::Copy => badged(Kind::Badge),
        Shape::NoDrop => badged(Kind::Ban),
        Shape::NotAllowed => bare(wagging()),
        Shape::Crosshair => rimmed(Kind::Reticle, 4.8, 45.0, 0.6),
        Shape::ZoomIn => lens(1.0, 1.0),
        Shape::ZoomOut => lens(0.8, 0.0),
        Shape::Kill => bare(aimed(gun(), -90.0, 0.92, ORIGIN)),
        Shape::Help => help(),
    }
}

const BUSY_FRAMES: usize = 48;
const BUSY_DELAY: u32 = 30;
/// Length of a resting frame in milliseconds, while the hand hovers.
const REST_STEP: u32 = 100;

/// Frames of an ordinary cursor and the order they play in: a busy loop, or
/// the segments of its idle pattern, each a hovering rest and perhaps a
/// gesture. Each distinct segment is posed once and played wherever the
/// pattern repeats it, its hover starting afresh so the repeats match.
pub fn frames(shape: Shape) -> (Vec<(Scene, u32)>, Vec<usize>) {
    if shape.is_busy() {
        let frames: Vec<_> =
            (0..BUSY_FRAMES).map(|i| (scene(shape, i as f32 / BUSY_FRAMES as f32), BUSY_DELAY)).collect();
        return (frames, (0..BUSY_FRAMES).collect());
    }
    let (hold, count) = idle::timing(shape);
    let rest = scene(shape, 0.0);
    let pattern = idle::pattern(shape);
    let mut segments: Vec<Option<usize>> = Vec::new();
    for segment in &pattern {
        if !segments.contains(segment) {
            segments.push(*segment);
        }
    }
    let (mut frames, mut spans) = (Vec::new(), Vec::new());
    for segment in &segments {
        let resting = match segment {
            Some(_) => hold,
            None => idle::BOB,
        };
        // A gesture with no rest before it starts at rest itself, so it loops.
        let gesture = segment.map_or(0..0, |_| usize::from(resting > 0)..count);
        let steps: Vec<(Option<f32>, u32)> = (0..resting / REST_STEP)
            .map(|_| (None, REST_STEP))
            .chain(gesture.map(|i| (Some(i as f32 / count as f32), idle::STEP)))
            .collect();
        let length: u32 = steps.iter().map(|(_, delay)| delay).sum();
        spans.push((frames.len(), steps.len()));
        let mut time = 0;
        for (u, delay) in steps {
            let posed = match (u, segment) {
                (Some(u), Some(variant)) => idle::gesture(shape, rest.clone(), u, *variant),
                _ => rest.clone(),
            };
            frames.push((idle::hover(shape, posed, time as f32 / length as f32, length), delay));
            time += delay;
        }
    }
    let order = pattern
        .iter()
        .flat_map(|segment| {
            let (start, len) = spans[segments.iter().position(|s| s == segment).expect("segment posed")];
            start..start + len
        })
        .collect();
    (frames, order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rig::blend;

    #[test]
    fn reversed_transition_matches_forward_transition() {
        for a in Shape::ALL {
            for b in Shape::ALL {
                let (first, last) = (scene(a, 0.0), scene(b, 0.0));
                for step in 0..=20 {
                    let t = step as f32 / 20.0;
                    let (forward, backward) = (blend(&first, &last, t), blend(&last, &first, 1.0 - t));
                    assert!((forward.hand.at - backward.hand.at).length() < 1e-4, "{a:?} -> {b:?} at {t}");
                    assert!((forward.hand.turn - backward.hand.turn).abs() < 1e-3);
                    let sizes = |scene: &Scene| {
                        let mut sizes: Vec<f32> = scene.props.iter().map(|(_, p)| p.size).collect();
                        sizes.sort_by(f32::total_cmp);
                        sizes
                    };
                    for (x, y) in sizes(&forward).into_iter().zip(sizes(&backward)) {
                        assert!((x - y).abs() < 1e-4, "{a:?} -> {b:?} at {t}");
                    }
                }
            }
        }
    }

    #[test]
    fn held_thumbs_point_up() {
        for angle in [0.0, 45.0, 90.0, 135.0, 180.0, -45.0, -90.0, 225.0] {
            let hand = holding(ORIGIN, angle, 1.0);
            let thumb = hand.place(V(-1.0, 0.0, 0.0)) - hand.place(ORIGIN);
            assert!(thumb.1 <= 1e-4, "{angle}");
        }
    }

    #[test]
    fn walking_feet_stay_planted() {
        // The index leg stands through the first half of the stride, the middle
        // leg through the second.
        for (leg, start) in [(0, 0), (1, 24)] {
            let heights: Vec<f32> = (start..start + 24)
                .map(|i| scene(Shape::Wait, i as f32 / 48.0).hand)
                .map(|hand| hand.place(crate::rig::fingertip(&hand, leg)).1)
                .collect();
            let spread = heights.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                - heights.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            assert!(spread < 0.15, "leg {leg} lifts by {spread} while planted");
        }
    }

    #[test]
    fn pointing_cursors_touch_the_hotspot() {
        for shape in [Shape::Default, Shape::Pointer] {
            let hand = scene(shape, 0.0).hand;
            let tip = hand.place(index_tip(&hand));
            assert!(tip.0.abs() < 1e-4 && tip.1.abs() < 1e-4, "{shape:?}");
        }
    }
}
