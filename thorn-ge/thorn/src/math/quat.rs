use super::{Mat4, Vec3, named_indices::*};
use std::ops::Mul;


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat
{
    pub r: f32,
    pub i: f32,
    pub j: f32,
    pub k: f32,
}


impl Default for Quat
{
    fn default() -> Self
    {
        Self::new()
    }
}


impl Quat
{
    pub fn new() -> Self
    {
        Self {
            r: 1.0,
            i: 0.0,
            j: 0.0,
            k: 0.0,
        }
    }

    pub fn from_euler(axis: Vec3, angle: f32) -> Self
    {
        let half = angle * 0.5;
        let sin = half.sin();
        let cos = half.cos();

        Self {
            r: cos,
            i: sin * axis[X],
            j: sin * axis[Y],
            k: sin * axis[Z],
        }
    }

    pub fn slerp(&self, to: Self, amount: f32) -> Self
    {
        const THRESHOLD: f32 = 0.9995;

        let mut q1 = self.norm();
        let q2 = to.norm();
        let mut dot = q1.dot(q2);

        if dot < 0.0
        {
            q1.r = -q1.r;
            q1.i = -q1.i;
            q1.j = -q1.j;
            q1.k = -q1.k;
            dot = -dot;
        }

        if dot > THRESHOLD
        {
            return Self {
                r: q1.r + ((q2.r - q1.r) * amount),
                i: q1.i + ((q2.i - q1.i) * amount),
                j: q1.j + ((q2.j - q1.j) * amount),
                k: q1.k + ((q2.k - q1.k) * amount),
            }
            .norm();
        }

        let t0 = dot.acos();
        let t = t0 * amount;
        let sin_t = t.sin();
        let sin_t0 = t0.sin();

        let s0 = t.cos() - dot * sin_t / sin_t0;
        let s1 = sin_t / sin_t0;

        Self {
            r: (q1.r * s0) + (q2.r * s1),
            i: (q1.i * s0) + (q2.i * s1),
            j: (q1.j * s0) + (q2.j * s1),
            k: (q1.k * s0) + (q2.k * s1),
        }
    }

    pub fn length(&self) -> f32
    {
        (self.r * self.r + self.i * self.i + self.j * self.j + self.k * self.k).sqrt()
    }

    pub fn norm(&self) -> Self
    {
        let n = self.length();

        Self {
            r: self.r / n,
            i: self.i / n,
            j: self.j / n,
            k: self.k / n,
        }
    }

    pub fn normalize(&mut self)
    {
        *self = self.norm();
    }

    pub fn conjugate(&self) -> Self
    {
        Self {
            r: self.r,
            i: -self.i,
            j: -self.j,
            k: -self.k,
        }
    }

    pub fn inverse(&self) -> Self
    {
        self.conjugate().norm()
    }

    pub fn dot(&self, other: Quat) -> f32
    {
        self.r * other.r + self.i * other.i + self.j * other.j + self.k * other.k
    }

    pub fn to_rotation_matrix(&self, center: Vec3) -> Mat4
    {
        let q = self.norm(); //TODO: Should this even be normalized?
        let mut ret = Mat4::new();

        ret[0][0] = q.i * q.i - q.j * q.j - q.k * q.k + q.r * q.r;
        ret[0][1] = 2.0 * (q.i * q.j + q.k * q.r);
        ret[0][2] = 2.0 * (q.i * q.k - q.j * q.r);
        ret[0][3] =
            center[X] - center[X] * ret[0][0] - center[Y] * ret[0][1] - center[Z] * ret[0][2];

        ret[1][0] = 2.0 * (q.i * q.j - q.k * q.r);
        ret[1][1] = -(q.i * q.i) + q.j * q.j - q.k * q.k + q.r * q.r;
        ret[1][2] = 2.0 * (q.j * q.k + q.i * q.r);
        ret[1][3] =
            center[Y] - center[X] * ret[1][0] - center[Y] * ret[1][1] - center[Z] * ret[1][2];

        ret[2][0] = 2.0 * (q.i * q.k + q.j * q.r);
        ret[2][1] = 2.0 * (q.j * q.k - q.i * q.r);
        ret[2][2] = -(q.i * q.i) - q.j * q.j + q.k * q.k + q.r * q.r;
        ret[1][3] =
            center[Z] - center[X] * ret[2][0] - center[Y] * ret[2][1] - center[Z] * ret[2][2];

        ret
    }
}


impl Mul<Quat> for Quat
{
    type Output = Quat;

    fn mul(self, rhs: Quat) -> Self::Output
    {
        let mut ret = Quat::new();

        ret.r = self.r * rhs.r - self.i * rhs.i - self.j * rhs.j - self.k * rhs.k;
        ret.i = self.r * rhs.i + self.i * rhs.r + self.j * rhs.k - self.k * rhs.j;
        ret.i = self.r * rhs.j - self.i * rhs.k + self.j * rhs.r + self.k * rhs.i;
        ret.i = self.r * rhs.k + self.i * rhs.j - self.j * rhs.i + self.k * rhs.r;

        ret
    }
}
