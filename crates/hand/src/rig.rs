//! The glove and its props as posed parameters, the reversible blend between
//! poses, and the solids a pose builds.
//!
//! The glove is a right hand. In its own frame the palm is centered on the
//! origin, the fingers reach toward -y, the back faces +z and the thumb sits
//! on the -x side.

use std::f32::consts::PI;

use cursor_core::Palette;
use cursor_core::color::{Color, hex, mix as mix_color};
use cursor_core::motion::{ease, lerp, window};

use crate::solid::{BODIES, Blend, Form, Model, Solid};
use crate::vector::{Rot, V};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// Center of the palm.
    pub at: V,
    pub size: f32,
    /// Degrees: turn about the hand's long axis (applied first; 180 shows the
    /// palm), then tip of the fingers away from the viewer, then roll on screen.
    pub turn: f32,
    pub tip: f32,
    pub roll: f32,
    /// Index to pinky: sideways spread in degrees, and curl from 0 (straight)
    /// to 1 (fist).
    pub spread: [f32; 4],
    pub curl: [f32; 4],
    /// Extra bend of each finger joint, knuckle to tip, in degrees.
    pub flex: [[f32; 3]; 4],
    /// Thumb swing away from the palm and fold under it in degrees, then its
    /// bend from 0 to 1.
    pub thumb: [f32; 3],
}

impl Pose {
    fn rot(&self) -> Rot {
        Rot::z(self.roll).then(&Rot::x(self.tip)).then(&Rot::y(self.turn))
    }

    /// World position of a point in the hand's frame.
    pub fn place(&self, local: V) -> V {
        self.at + self.rot().apply(local * self.size)
    }

    /// The pose moved so `local` lands on `target`.
    pub fn pinned(self, local: V, target: V) -> Pose {
        Pose { at: self.at + target - self.place(local), ..self }
    }

    fn mix(&self, other: &Pose, t: f32) -> Pose {
        let each = |a: [f32; 4], b: [f32; 4]| std::array::from_fn(|i| lerp(a[i], b[i], t));
        Pose {
            at: self.at.mix(other.at, t),
            size: lerp(self.size, other.size, t),
            turn: lerp(self.turn, other.turn, t),
            tip: lerp(self.tip, other.tip, t),
            roll: lerp(self.roll, other.roll, t),
            spread: each(self.spread, other.spread),
            curl: each(self.curl, other.curl),
            flex: std::array::from_fn(|i| std::array::from_fn(|j| lerp(self.flex[i][j], other.flex[i][j], t))),
            thumb: std::array::from_fn(|i| lerp(self.thumb[i], other.thumb[i], t)),
        }
    }
}

const KNUCKLES: [V; 4] = [V(-2.75, -3.5, 0.25), V(-0.92, -3.85, 0.3), V(0.92, -3.7, 0.25), V(2.65, -3.25, 0.15)];
const LENGTHS: [[f32; 3]; 4] = [[3.0, 2.08, 1.75], [3.2, 2.2, 1.8], [3.0, 2.0, 1.7], [2.4, 1.6, 1.4]];
const WIDTHS: [f32; 4] = [1.15, 1.2, 1.12, 0.98];
/// Radius at each joint past the knuckle, tip last, as a share of the width.
const TAPER: [f32; 3] = [0.92, 0.84, 0.72];
/// Bend of each joint at full curl, in degrees.
const BEND: [f32; 3] = [85.0, 100.0, 70.0];
/// Where the thumb's first bone starts, low on the palm by the wrist, and
/// where it ends when the thumb is at rest.
const THUMB_BASE: V = V(-1.9, 3.4, -0.4);
const THUMB_ROOT: V = V(-3.1, 0.9, -0.5);
/// Center of the loop the fingers close around when holding a bar, in the
/// hand's frame.
pub const GRIP: V = V(0.0, -2.52, -3.33);

/// Joints of finger `i` as (center, radius), knuckle to tip, in the hand's
/// frame.
fn finger(pose: &Pose, i: usize) -> [(V, f32); 4] {
    let (s, c) = pose.spread[i].to_radians().sin_cos();
    let reach = V(s, -c, 0.0);
    let width = WIDTHS[i];
    let mut joints = [(KNUCKLES[i], width); 4];
    let mut angle = 0.0f32;
    for j in 0..3 {
        angle += pose.curl[i] * BEND[j] + pose.flex[i][j];
        let (s, c) = angle.to_radians().sin_cos();
        let dir = reach * c - V::Z * s;
        let radius = width * TAPER[j];
        joints[j + 1] = (joints[j].0 + dir * LENGTHS[i][j], radius);
    }
    joints
}

/// Joints of the thumb, wrist to tip: its first bone swings partway with
/// the thumb along the edge of the palm, then two jointed segments bend
/// across it.
fn thumb(pose: &Pose) -> [(V, f32); 4] {
    let [out, fold, bend] = pose.thumb;
    let (so, co) = out.to_radians().sin_cos();
    let (sf, cf) = fold.to_radians().sin_cos();
    let first = V(-so, -co, 0.0) * cf - V::Z * sf;
    let rest = THUMB_ROOT - THUMB_BASE;
    let bone = (rest.normalized().mix(first, 0.5)).normalized() * rest.length();
    let across = V(1.0, 0.0, -0.7).normalized();
    let side = (across - first * across.dot(first)).normalized();
    let bent = |degrees: f32| {
        let (s, c) = (bend * degrees).to_radians().sin_cos();
        first * c + side * s
    };
    let knuckle = THUMB_BASE + bone;
    let joint = knuckle + bent(25.0) * 2.4;
    [(THUMB_BASE, 1.4), (knuckle, 1.12), (joint, 1.02), (joint + bent(75.0) * 1.9, 0.86)]
}

/// The very end of finger `i`, in the hand's frame.
pub fn fingertip(pose: &Pose, i: usize) -> V {
    let joints = finger(pose, i);
    let (end, radius) = joints[3];
    end + (end - joints[2].0).normalized() * radius
}

/// The very end of the index finger, in the hand's frame.
pub fn index_tip(pose: &Pose) -> V {
    fingertip(pose, 0)
}

/// Midpoint of the index and middle knuckles, where the walking legs meet
/// the body, in world space.
pub fn hips(pose: &Pose) -> V {
    pose.place((KNUCKLES[0] + KNUCKLES[1]) * 0.5)
}

/// Length of finger `i` held straight, knuckle to the very tip.
pub fn leg_length(i: usize) -> f32 {
    LENGTHS[i].iter().sum::<f32>() + WIDTHS[i] * TAPER[2]
}

/// Knuckle of finger `i`, in the hand's frame.
pub fn reach_root(i: usize) -> V {
    KNUCKLES[i]
}

/// Joint bends of finger `i` that put its tip on `target` as seen on screen,
/// or as near as it reaches, with the finger otherwise uncurled. The tip
/// joint follows the middle one as real fingers do, and that one only bends
/// toward the palm, like a knee.
pub fn reach(pose: &Pose, i: usize, target: V) -> [f32; 3] {
    const TIP: f32 = 0.7;
    let (s, c) = pose.spread[i].to_radians().sin_cos();
    let along = V(s, -c, 0.0);
    let end = WIDTHS[i] * TAPER[2];
    // The finger bends in the plane of its reach and the palm's normal.
    let tip = |hip: f32, knee: f32| {
        let mut angle = 0.0f32;
        let (u, w) = [hip, knee, TIP * knee].into_iter().enumerate().fold((0.0, 0.0), |(u, w), (j, bend)| {
            angle += bend;
            let length = LENGTHS[i][j] + if j == 2 { end } else { 0.0 };
            let (s, c) = angle.to_radians().sin_cos();
            (u + c * length, w + s * length)
        });
        let p = pose.place(KNUCKLES[i] + along * u - V::Z * w);
        (p.0, p.1)
    };
    let miss = |(x, y): (f32, f32)| (target.0 - x).hypot(target.1 - y);
    // A coarse search over the joint limits, then damped least squares on
    // the two free angles, in degrees.
    let coarse = (0..=40).flat_map(|h| (0..=24).map(move |k| (-60.0 + 5.0 * h as f32, 5.0 * k as f32)));
    let (mut hip, mut knee) = coarse.min_by(|a, b| miss(tip(a.0, a.1)).total_cmp(&miss(tip(b.0, b.1)))).unwrap();
    for _ in 0..40 {
        let p = tip(hip, knee);
        let error = (target.0 - p.0, target.1 - p.1);
        let (h, k) = (tip(hip + 0.5, knee), tip(hip, knee + 0.5));
        let (a, b, c, d) = ((h.0 - p.0) / 0.5, (k.0 - p.0) / 0.5, (h.1 - p.1) / 0.5, (k.1 - p.1) / 0.5);
        let (m00, m01, m11) = (a * a + c * c + 1e-4, a * b + c * d, b * b + d * d + 1e-4);
        let (g0, g1) = (a * error.0 + c * error.1, b * error.0 + d * error.1);
        let det = m00 * m11 - m01 * m01;
        hip = (hip + (m11 * g0 - m01 * g1) / det).clamp(-60.0, 140.0);
        knee = (knee + (m00 * g1 - m01 * g0) / det).clamp(0.0, 120.0);
    }
    [hip, knee, TIP * knee]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// Text caret: a stem with serifs, lying along x like the arrows so the
    /// two morph end to end.
    Beam,
    /// Column divider: a bar along x, split by a double arrow across it,
    /// so it lines up with the caret.
    Columns,
    /// Double arrow; `split` grows a bar across its middle.
    Arrows {
        split: f32,
    },
    /// Ban sign.
    Ban,
    /// Ball with a plus on it.
    Badge,
    /// Magnifier with a minus; `plus` grows it into a plus.
    Lens {
        plus: f32,
    },
    Reticle,
    /// Ball circled by two rings.
    Gyro,
}

impl Kind {
    fn mix(self, other: Kind, t: f32) -> Option<Kind> {
        Some(match (self, other) {
            (Kind::Arrows { split: a }, Kind::Arrows { split: b }) => Kind::Arrows { split: lerp(a, b, t) },
            (Kind::Lens { plus: a }, Kind::Lens { plus: b }) => Kind::Lens { plus: lerp(a, b, t) },
            (a, b) if a == b => a,
            _ => return None,
        })
    }
}

/// Turn and tip of the glove when it holds a prop, which sets how held props
/// sit in its frame.
pub const HOLD_TURN: f32 = 180.0;
pub const HOLD_TIP: f32 = -50.0;

/// Where a held prop sits in the hand: the point in the prop's own frame
/// that lies on `GRIP`, and the prop's roll beyond the hand's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hold {
    pub offset: V,
    pub twist: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prop {
    pub kind: Kind,
    /// Another kind this prop is turning into, and how far.
    pub morph: Option<(Kind, f32)>,
    /// Free props sit at `at` and `roll`; held ones follow the hand instead.
    pub held: Option<Hold>,
    pub at: V,
    /// Degrees: spin about the prop's vertical axis, then roll on screen.
    pub spin: f32,
    pub roll: f32,
    pub size: f32,
    /// Opacity, as props fade in and out.
    pub fade: f32,
}

pub fn prop(kind: Kind, at: V, roll: f32) -> Prop {
    Prop { kind, morph: None, held: None, at, spin: 0.0, roll, size: 1.0, fade: 1.0 }
}

impl Prop {
    pub fn sized(self, size: f32) -> Prop {
        Prop { size, ..self }
    }

    pub fn held(self, hold: Hold) -> Prop {
        Prop { held: Some(hold), ..self }
    }

    /// Props of one kind blend their parameters; others morph into each other.
    fn mix(&self, other: &Prop, t: f32) -> Prop {
        let (kind, morph) = match self.kind.mix(other.kind, t) {
            Some(kind) => (kind, None),
            None => (self.kind, Some((other.kind, t))),
        };
        let held = match (self.held, other.held) {
            (Some(a), Some(b)) => Some(Hold { offset: a.offset.mix(b.offset, t), twist: lerp(a.twist, b.twist, t) }),
            (a, b) => a.or(b),
        };
        Prop {
            kind,
            morph,
            held,
            at: self.at.mix(other.at, t),
            spin: lerp(self.spin, other.spin, t),
            roll: lerp(self.roll, other.roll, t),
            size: lerp(self.size, other.size, t),
            fade: lerp(self.fade, other.fade, t),
        }
    }

    /// Faded out by `s` from 0 to 1, held props still riding in the hand.
    fn gone(self, s: f32) -> Prop {
        Prop { fade: self.fade * (1.0 - s), ..self }
    }

    /// Center and orientation, following `hand` when held.
    fn frame(&self, hand: &Pose) -> (V, Rot) {
        let spin = Rot::y(self.spin);
        match self.held {
            None => (self.at, Rot::z(self.roll).then(&spin)),
            Some(hold) => {
                let seat = Rot::y(-HOLD_TURN).then(&Rot::x(-HOLD_TIP)).then(&Rot::z(hold.twist));
                let rot = hand.rot().then(&seat).then(&spin);
                (hand.place(GRIP) - rot.apply(hold.offset * self.size), rot)
            }
        }
    }
}

pub type Props = Vec<(&'static str, Prop)>;

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub hand: Pose,
    pub props: Props,
}

/// Mirrored windows for props on only one side, so playing a transition
/// backwards is exactly the opposite transition. Held props fade in as the
/// hand closes on them, and out as it lets go.
const LEAVE: (f32, f32) = (0.0, 0.35);
const ENTER: (f32, f32) = (0.65, 1.0);
const LEAVE_HELD: (f32, f32) = (0.2, 0.55);
const ENTER_HELD: (f32, f32) = (0.45, 0.8);
/// The hand opens over the first window and closes over the second when it
/// takes up or lets go of a held prop.
const OPENING: (f32, f32) = (0.0, 0.4);
const CLOSING: (f32, f32) = (0.45, 1.0);

/// The fingers of `pose` opened into a loose claw and its thumb swung out,
/// keeping its fold so it closes back around the outside of what the hand
/// takes.
fn opened(pose: &Pose) -> Pose {
    Pose { curl: [0.22; 4], flex: [[0.0; 3]; 4], thumb: [pose.thumb[0] + 30.0, pose.thumb[1], 0.1], ..*pose }
}

/// The scene at progress `t` in [0, 1] from `a` to `b`, props in key order so
/// both directions draw identically. Props on one side fade in or out, held
/// ones turning with the hand as they do; a hand taking up a held prop opens,
/// then closes on it as it fades in between the fingers, and one letting go
/// does the same in reverse.
pub fn blend(a: &Scene, b: &Scene, t: f32) -> Scene {
    let holds = |scene: &Scene| scene.props.iter().any(|(_, p)| p.held.is_some());
    let mut hand = a.hand.mix(&b.hand, ease(t));
    if holds(a) != holds(b) {
        let (from, to, s) = match t < 0.5 {
            true => (a.hand, opened(&b.hand), ease(window(t, OPENING))),
            false => (opened(&a.hand), b.hand, ease(window(t, CLOSING))),
        };
        let fingers = from.mix(&to, s);
        hand = Pose { curl: fingers.curl, flex: fingers.flex, thumb: fingers.thumb, ..hand };
    }
    let flip = ((a.hand.turn - b.hand.turn).abs() / 180.0).min(1.0);
    let hand = swooped(hand, (PI * t).sin(), flip);
    let find = |scene: &Scene, key: &str| scene.props.iter().find(|(k, _)| *k == key).map(|(_, p)| *p);
    let mut keys: Vec<&'static str> = a.props.iter().chain(&b.props).map(|(key, _)| *key).collect();
    keys.sort_unstable();
    keys.dedup();
    let props = keys
        .into_iter()
        .map(|key| {
            let windows = |p: &Prop| if p.held.is_some() { (LEAVE_HELD, ENTER_HELD) } else { (LEAVE, ENTER) };
            let prop = match (find(a, key), find(b, key)) {
                (Some(x), Some(y)) => x.mix(&y, ease(t)),
                (Some(x), None) => x.gone(ease(window(t, windows(&x).0))),
                (None, Some(y)) => y.gone(1.0 - ease(window(t, windows(&y).1))),
                (None, None) => unreachable!(),
            };
            (key, prop)
        })
        .collect();
    Scene { hand, props }
}

/// Mid-flight flourish: by `lift` from 0 to 1, the hand rises toward the
/// viewer, tips its fingers forward and loosens its fingers. A hand turning
/// over by `flip` of a half turn floats further forward as it does.
fn swooped(pose: Pose, lift: f32, flip: f32) -> Pose {
    Pose {
        size: pose.size * (1.0 + (0.06 + 0.1 * flip) * lift),
        tip: pose.tip - 22.0 * lift,
        curl: pose.curl.map(|c| c - 0.1 * lift * c.max(0.3)),
        ..pose
    }
}

const PALM: u8 = 1;
const CUFF: u8 = 2;
const FINGER: u8 = 3;
const THUMB: u8 = 7;
/// First part of props; each prop numbers its pieces from its own base.
const PROP: u8 = 16;

/// Solids placed in a frame, all of one body.
struct Builder<'a> {
    solids: &'a mut Vec<Solid>,
    at: V,
    rot: Rot,
    size: f32,
    body: u8,
    melt: f32,
    joins: u8,
}

impl Builder<'_> {
    fn push(&mut self, form: Form, at: V, rot: Rot, (light, dark): (Color, Color), part: u8) {
        let k = self.size;
        let form = match form {
            Form::Cone { a, ra, b, rb } => {
                let place = |p: V| self.at + self.rot.apply(p * k);
                Form::Cone { a: place(a), ra: ra * k, b: place(b), rb: rb * k }
            }
            Form::Slab { half, round } => Form::Slab { half: half * k, round: round * k },
            Form::Ring { major, minor } => Form::Ring { major: major * k, minor: minor * k },
            Form::Frustum { half, r0, r1, round } => {
                Form::Frustum { half: half * k, r0: r0 * k, r1: r1 * k, round: round * k }
            }
            Form::Ellipsoid { radii } => Form::Ellipsoid { radii: radii * k },
        };
        let (at, rot) = (self.at + self.rot.apply(at * k), self.rot.then(&rot));
        let (body, melt, joins) = (self.body, self.melt * k, self.joins);
        self.solids.push(Solid { form, at, rot, light, dark, part, joins, body, melt });
    }

    fn cone(&mut self, a: V, ra: f32, b: V, rb: f32, tones: (Color, Color), part: u8) {
        self.push(Form::Cone { a, ra, b, rb }, V::default(), Rot::IDENTITY, tones, part);
    }

    fn capsule(&mut self, a: V, b: V, r: f32, tones: (Color, Color), part: u8) {
        self.cone(a, r, b, r, tones, part);
    }

    fn ball(&mut self, at: V, r: f32, tones: (Color, Color), part: u8) {
        self.cone(at, r, at, r, tones, part);
    }

    /// Torus facing the viewer.
    fn hoop(&mut self, at: V, major: f32, minor: f32, tones: (Color, Color), part: u8) {
        self.push(Form::Ring { major, minor }, at, Rot::x(90.0), tones, part);
    }
}

/// Lit and shaded tones of a prop color.
fn tones(colors: &Palette, color: Color) -> (Color, Color) {
    (color, mix_color(color, colors["ink"], 0.32))
}

fn glove(pose: &Pose, colors: &Palette, solids: &mut Vec<Solid>) {
    let tones = (colors["glove"], colors["shade"]);
    let (at, rot, size) = (pose.at, pose.rot(), pose.size);
    let mut hand = Builder { solids, at, rot, size, body: 0, melt: 0.45, joins: 0 };
    // The palm: a thin core under a gently domed body, a ridge running to
    // each knuckle across the back, pads at the heels of the thumb and pinky
    // on the palm side, and an oval wrist, all melted smooth.
    let oval = |radii: V| Form::Ellipsoid { radii };
    hand.melt = 0.9;
    hand.push(Form::Slab { half: V(3.7, 3.2, 0.9), round: 0.85 }, V(0.0, -0.5, 0.0), Rot::IDENTITY, tones, PALM);
    hand.push(oval(V(3.9, 4.3, 1.55)), V(0.0, 0.4, 0.05), Rot::IDENTITY, tones, PALM);
    for knuckle in KNUCKLES {
        hand.capsule(V(knuckle.0 * 0.55, 3.0, 0.4), V(knuckle.0, knuckle.1 + 0.4, 0.6), 0.95, tones, PALM);
    }
    hand.push(oval(V(1.4, 2.0, 1.0)), V(-2.2, 1.4, -0.7), Rot::z(25.0), tones, PALM);
    // The outer edge leans in from the pinky knuckle to the wrist.
    hand.push(oval(V(1.3, 2.9, 1.1)), V(2.3, 1.3, -0.15), Rot::z(16.0), tones, PALM);
    hand.push(oval(V(2.6, 2.2, 1.5)), V(0.0, 4.0, 0.0), Rot::IDENTITY, tones, PALM);
    hand.joins = PALM;
    // Each finger melts into the palm at its base but keeps crisp joints.
    let chains = (0..4).map(|i| (finger(pose, i).to_vec(), FINGER + i as u8));
    for (joints, part) in chains.chain([(thumb(pose).to_vec(), THUMB)]) {
        for (j, pair) in joints.windows(2).enumerate() {
            // The thumb's bone swells into the heel of the palm.
            hand.melt = match (j, part) {
                (0, THUMB) => 1.1,
                (0, _) => 0.45,
                _ => 0.08,
            };
            hand.cone(pair[0].0, pair[0].1, pair[1].0, pair[1].1, tones, part);
        }
    }
    // A web of skin from partway up the thumb to the index knuckle.
    let [_, knuckle, joint, _] = thumb(pose);
    hand.melt = 0.9;
    hand.cone(knuckle.0.mix(joint.0, 0.45), 0.6, KNUCKLES[0] + V(0.2, 0.9, -0.3), 0.75, tones, PALM);
    let mut cuff = Builder { body: 1, melt: 0.2, joins: 0, ..hand };
    let flare = Form::Frustum { half: 0.9, r0: 2.8, r1: 3.2, round: 0.35 };
    cuff.push(flare, V(0.0, 5.5, 0.0), Rot::IDENTITY, tones, CUFF);
    cuff.push(Form::Ring { major: 3.05, minor: 0.5 }, V(0.0, 6.4, 0.0), Rot::IDENTITY, tones, CUFF);
}

/// Double arrow along x, with a bar along y grown by `split`.
fn arrows(colors: &Palette, mut b: Builder, split: f32, base: u8) {
    let t = tones(colors, colors["resize"]);
    b.capsule(V(-6.0, 0.0, 0.0), V(6.0, 0.0, 0.0), 0.95, t, base);
    for side in [-1.0, 1.0] {
        let head = Form::Frustum { half: 1.6, r0: 2.4, r1: 0.25, round: 0.3 };
        b.push(head, V(7.4 * side, 0.0, 0.0), Rot::z(-90.0 * side), t, base);
    }
    let reach = 6.0 * split;
    b.capsule(V(0.0, -reach, 0.0), V(0.0, reach, 0.0), split.min(0.25) * 4.0, t, base);
}

fn piece(colors: &Palette, prop: &Prop, (at, rot): (V, Rot), body: u8, solids: &mut Vec<Solid>) {
    let mut b = Builder { solids, at, rot, size: prop.size, body, melt: 0.15, joins: 0 };
    let base = PROP + 4 * body;
    let white = (colors["glove"], colors["shade"]);
    match prop.kind {
        Kind::Beam => {
            let t = tones(colors, colors["text"]);
            b.capsule(V(-6.4, 0.0, 0.0), V(6.4, 0.0, 0.0), 0.9, t, base);
            for x in [-6.6, 6.6] {
                b.capsule(V(x, -2.4, 0.0), V(x, 2.4, 0.0), 0.85, t, base);
            }
        }
        Kind::Arrows { split } => arrows(colors, b, split, base),
        Kind::Columns => arrows(colors, Builder { rot: b.rot.then(&Rot::z(90.0)), ..b }, 1.0, base),
        Kind::Ban => {
            let t = tones(colors, colors["refuse"]);
            b.hoop(V::default(), 5.4, 1.05, t, base);
            b.capsule(V(-3.8, -3.8, 0.0), V(3.8, 3.8, 0.0), 1.05, t, base);
        }
        Kind::Badge => {
            b.ball(V::default(), 2.8, tones(colors, colors["copy"]), base);
            b.capsule(V(-1.5, 0.0, 2.35), V(1.5, 0.0, 2.35), 0.5, white, base);
            b.capsule(V(0.0, -1.5, 2.35), V(0.0, 1.5, 2.35), 0.5, white, base);
        }
        Kind::Lens { plus } => {
            let t = tones(colors, colors["zoom"]);
            let glass = (mix_color(colors["zoom"], hex(0xffffff), 0.78), mix_color(colors["zoom"], hex(0xffffff), 0.5));
            b.hoop(V::default(), 4.5, 0.95, t, base);
            let disc = Form::Frustum { half: 0.3, r0: 4.0, r1: 4.0, round: 0.25 };
            b.push(disc, V::default(), Rot::x(90.0), glass, base + 1);
            let (s, c) = (PI / 4.0).sin_cos();
            b.cone(V(c * 5.2, s * 5.2, 0.0), 0.9, V(c * 10.5, s * 10.5, 0.0), 1.0, t, base + 2);
            b.capsule(V(-2.0, 0.0, 0.5), V(2.0, 0.0, 0.5), 0.55, t, base + 3);
            b.capsule(V(0.0, -2.0 * plus, 0.5), V(0.0, 2.0 * plus, 0.5), 0.55 * plus.min(0.25) * 4.0, t, base + 3);
        }
        Kind::Reticle => {
            let t = tones(colors, colors["crosshair"]);
            b.hoop(V::default(), 4.8, 0.6, t, base);
            // Ticks run out from the ring, joined to it.
            for angle in [0.0f32, 90.0, 180.0, 270.0] {
                let (s, c) = angle.to_radians().sin_cos();
                b.capsule(V(c * 4.8, s * 4.8, 0.0), V(c * 7.4, s * 7.4, 0.0), 0.62, t, base);
            }
            b.ball(V::default(), 0.85, t, base);
        }
        Kind::Gyro => {
            let t = tones(colors, colors["busy"]);
            b.ball(V::default(), 1.5, t, base);
            for (part, tilt) in [(base + 1, 62.0), (base + 2, -62.0)] {
                b.push(Form::Ring { major: 3.0, minor: 0.5 }, V::default(), Rot::z(tilt), t, part);
            }
        }
    }
}

/// Every solid of a scene, with morphing props as blends of two bodies.
pub fn model(colors: &Palette, scene: &Scene) -> Model {
    let mut model = Model::default();
    glove(&scene.hand, colors, &mut model.solids);
    let mut body = 2;
    for (_, prop) in scene.props.iter().filter(|(_, p)| p.size > 1e-3 && p.fade > 1e-3) {
        let frame = prop.frame(&scene.hand);
        piece(colors, prop, frame, body, &mut model.solids);
        if let Some((kind, t)) = prop.morph {
            piece(colors, &Prop { kind, ..*prop }, frame, body + 1, &mut model.solids);
            model.blends.push(Blend { from: body, to: body + 1, t });
            body += 1;
        }
        body += 1;
    }
    assert!(body as usize <= BODIES, "too many props");
    model
}
