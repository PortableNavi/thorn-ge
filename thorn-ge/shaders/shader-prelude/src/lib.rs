#![cfg_attr(target_arch = "spirv", no_std)]
#![feature(asm_experimental_arch)]

use core::f32::consts::PI;
use core::ops::{Add, Mul, Sub};
use spirv_std::glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

pub fn saturate(x: f32) -> f32
{
    x.max(0.0).min(1.0)
}

/// Based on: https://seblagarde.wordpress.com/2014/12/01/inverse-trigonometric-functions-gpu-optimization-for-amd-gcn-architecture/
pub fn acos_approx(v: f32) -> f32
{
    let x = v.abs();
    let mut res = -0.155972 * x + 1.56467; // p(x)
    res *= (1.0f32 - x).sqrt();

    if v >= 0.0
    {
        res
    }
    else
    {
        PI - res
    }
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32
{
    // Scale, bias and saturate x to 0..1 range
    let x = saturate((x - edge0) / (edge1 - edge0));
    // Evaluate polynomial
    x * x * (3.0 - 2.0 * x)
}

pub fn mix<X: Copy + Mul<A, Output = X> + Add<Output = X> + Sub<Output = X>, A: Copy>(
    x: X,
    y: X,
    a: A,
) -> X
{
    x - x * a + y * a
}

pub trait Clamp
{
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl Clamp for f32
{
    fn clamp(self, min: Self, max: Self) -> Self
    {
        self.max(min).min(max)
    }
}

pub trait FloatExt
{
    fn fract_gl(self) -> Self;
    fn rem_euclid(self, rhs: Self) -> Self;
    fn sign_gl(self) -> Self;
    fn step(self, x: Self) -> Self;
}

impl FloatExt for f32
{
    fn fract_gl(self) -> f32
    {
        self - self.floor()
    }

    fn rem_euclid(self, rhs: f32) -> f32
    {
        let r = self % rhs;
        if r < 0.0
        {
            r + rhs.abs()
        }
        else
        {
            r
        }
    }

    fn sign_gl(self) -> f32
    {
        if self < 0.0
        {
            -1.0
        }
        else if self == 0.0
        {
            0.0
        }
        else
        {
            1.0
        }
    }

    fn step(self, x: f32) -> f32
    {
        if x < self
        {
            0.0
        }
        else
        {
            1.0
        }
    }
}

pub trait VecExt
{
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn powf_vec(self, p: Self) -> Self;
    fn sqrt(self) -> Self;
    fn ln(self) -> Self;
    fn step(self, other: Self) -> Self;
    fn sign_gl(self) -> Self;
}

impl VecExt for Vec2
{
    fn sin(self) -> Vec2
    {
        vec2(self.x.sin(), self.y.sin())
    }

    fn cos(self) -> Vec2
    {
        vec2(self.x.cos(), self.y.cos())
    }

    fn powf_vec(self, p: Vec2) -> Vec2
    {
        vec2(self.x.powf(p.x), self.y.powf(p.y))
    }

    fn sqrt(self) -> Vec2
    {
        vec2(self.x.sqrt(), self.y.sqrt())
    }

    fn ln(self) -> Vec2
    {
        vec2(self.x.ln(), self.y.ln())
    }

    fn step(self, other: Vec2) -> Vec2
    {
        vec2(self.x.step(other.x), self.y.step(other.y))
    }

    fn sign_gl(self) -> Vec2
    {
        vec2(self.x.sign_gl(), self.y.sign_gl())
    }
}

impl VecExt for Vec3
{
    fn sin(self) -> Vec3
    {
        vec3(self.x.sin(), self.y.sin(), self.z.sin())
    }

    fn cos(self) -> Vec3
    {
        vec3(self.x.cos(), self.y.cos(), self.z.cos())
    }

    fn powf_vec(self, p: Vec3) -> Vec3
    {
        vec3(self.x.powf(p.x), self.y.powf(p.y), self.z.powf(p.z))
    }

    fn sqrt(self) -> Vec3
    {
        vec3(self.x.sqrt(), self.y.sqrt(), self.z.sqrt())
    }

    fn ln(self) -> Vec3
    {
        vec3(self.x.ln(), self.y.ln(), self.z.ln())
    }

    fn step(self, other: Vec3) -> Vec3
    {
        vec3(
            self.x.step(other.x),
            self.y.step(other.y),
            self.z.step(other.z),
        )
    }

    fn sign_gl(self) -> Vec3
    {
        vec3(self.x.sign_gl(), self.y.sign_gl(), self.z.sign_gl())
    }
}

impl VecExt for Vec4
{
    fn sin(self) -> Vec4
    {
        vec4(self.x.sin(), self.y.sin(), self.z.sin(), self.w.sin())
    }

    fn cos(self) -> Vec4
    {
        vec4(self.x.cos(), self.y.cos(), self.z.cos(), self.w.cos())
    }

    fn powf_vec(self, p: Vec4) -> Vec4
    {
        vec4(
            self.x.powf(p.x),
            self.y.powf(p.y),
            self.z.powf(p.z),
            self.w.powf(p.w),
        )
    }

    fn sqrt(self) -> Vec4
    {
        vec4(self.x.sqrt(), self.y.sqrt(), self.z.sqrt(), self.w.sqrt())
    }

    fn ln(self) -> Vec4
    {
        vec4(self.x.ln(), self.y.ln(), self.z.ln(), self.w.ln())
    }

    fn step(self, other: Vec4) -> Vec4
    {
        vec4(
            self.x.step(other.x),
            self.y.step(other.y),
            self.z.step(other.z),
            self.w.step(other.w),
        )
    }

    fn sign_gl(self) -> Vec4
    {
        vec4(
            self.x.sign_gl(),
            self.y.sign_gl(),
            self.z.sign_gl(),
            self.w.sign_gl(),
        )
    }
}

pub fn discard()
{
    unsafe { spirv_std::arch::demote_to_helper_invocation() }
}
