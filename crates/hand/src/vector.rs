//! Small 3D vectors and rotations. Design units, x right, y down, z toward
//! the viewer.

use std::ops::{Add, Mul, Neg, Sub};

use cursor_core::motion::lerp;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V(pub f32, pub f32, pub f32);

impl V {
    pub const X: V = V(1.0, 0.0, 0.0);
    pub const Y: V = V(0.0, 1.0, 0.0);
    pub const Z: V = V(0.0, 0.0, 1.0);

    pub fn dot(self, o: V) -> f32 {
        self.0 * o.0 + self.1 * o.1 + self.2 * o.2
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> V {
        self * (1.0 / self.length().max(1e-6))
    }

    pub fn mix(self, o: V, t: f32) -> V {
        V(lerp(self.0, o.0, t), lerp(self.1, o.1, t), lerp(self.2, o.2, t))
    }

    pub fn max(self, v: f32) -> V {
        V(self.0.max(v), self.1.max(v), self.2.max(v))
    }

    pub fn abs(self) -> V {
        V(self.0.abs(), self.1.abs(), self.2.abs())
    }
}

impl Add for V {
    type Output = V;

    fn add(self, o: V) -> V {
        V(self.0 + o.0, self.1 + o.1, self.2 + o.2)
    }
}

impl Sub for V {
    type Output = V;

    fn sub(self, o: V) -> V {
        V(self.0 - o.0, self.1 - o.1, self.2 - o.2)
    }
}

impl Mul<f32> for V {
    type Output = V;

    fn mul(self, k: f32) -> V {
        V(self.0 * k, self.1 * k, self.2 * k)
    }
}

impl Neg for V {
    type Output = V;

    fn neg(self) -> V {
        self * -1.0
    }
}

/// Rotation matrix, stored by rows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rot([V; 3]);

impl Rot {
    pub const IDENTITY: Rot = Rot([V::X, V::Y, V::Z]);

    /// About the x axis; positive turns y toward z.
    pub fn x(degrees: f32) -> Rot {
        let (s, c) = degrees.to_radians().sin_cos();
        Rot([V::X, V(0.0, c, -s), V(0.0, s, c)])
    }

    /// About the y axis; positive turns z toward x.
    pub fn y(degrees: f32) -> Rot {
        let (s, c) = degrees.to_radians().sin_cos();
        Rot([V(c, 0.0, s), V::Y, V(-s, 0.0, c)])
    }

    /// About the view axis; positive turns x toward y, clockwise on screen.
    pub fn z(degrees: f32) -> Rot {
        let (s, c) = degrees.to_radians().sin_cos();
        Rot([V(c, -s, 0.0), V(s, c, 0.0), V::Z])
    }

    pub fn apply(&self, v: V) -> V {
        V(self.0[0].dot(v), self.0[1].dot(v), self.0[2].dot(v))
    }

    /// Applies the inverse rotation.
    pub fn undo(&self, v: V) -> V {
        let [a, b, c] = self.0;
        a * v.0 + b * v.1 + c * v.2
    }

    /// `self` after `other`.
    pub fn then(&self, other: &Rot) -> Rot {
        let columns = [V::X, V::Y, V::Z].map(|axis| self.apply(other.apply(axis)));
        Rot(std::array::from_fn(|row| {
            let pick = |v: V| [v.0, v.1, v.2][row];
            V(pick(columns[0]), pick(columns[1]), pick(columns[2]))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: V, b: V) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn rotations_compose_and_undo() {
        assert!(close(Rot::z(90.0).apply(V::X), V::Y));
        assert!(close(Rot::x(90.0).apply(V::Y), V::Z));
        assert!(close(Rot::y(90.0).apply(V::Z), V::X));
        let r = Rot::z(30.0).then(&Rot::x(-50.0)).then(&Rot::y(20.0));
        let v = V(1.0, -2.0, 0.5);
        assert!(close(r.undo(r.apply(v)), v));
        assert!(close(r.apply(v), Rot::z(30.0).apply(Rot::x(-50.0).apply(Rot::y(20.0).apply(v)))));
    }
}
