use std::{
    ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign},
    simd::{LaneCount, SupportedLaneCount, prelude::*},
};


#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Vector<const D: usize>(pub(super) Simd<f32, D>)
where
    LaneCount<D>: SupportedLaneCount;


impl<const D: usize> Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    pub const ZERO: Self = Self::splat(0.0);

    pub const fn splat(value: f32) -> Self
    {
        Self(Simd::splat(value))
    }

    pub fn as_slice(&self) -> &[f32]
    {
        self.as_ref()
    }

    #[inline]
    pub fn sum(&self) -> f32
    {
        self.0.reduce_sum()
    }

    #[inline]
    pub fn powu(&self, power: usize) -> Self
    {
        let mut ret = Self::splat(1.0);

        for _ in 0..power
        {
            ret *= self;
        }

        ret
    }

    #[inline]
    pub fn normalized(&mut self)
    {
        *self /= self.length();
    }

    #[inline]
    pub fn norm(&self) -> Self
    {
        *self / self.length()
    }

    #[inline]
    pub fn distance_sq(&self, other: Self) -> f32
    {
        (other - self).length_sq()
    }

    #[inline]
    pub fn distance(&self, other: Self) -> f32
    {
        (other - self).length()
    }

    #[inline]
    pub fn dot(&self, other: Self) -> f32
    {
        (*self * other).sum()
    }


    #[inline]
    pub fn length_sq(&self) -> f32
    {
        self.powu(2).sum()
    }

    #[inline]
    pub fn length(&self) -> f32
    {
        self.length_sq().sqrt()
    }
}


impl<const D: usize> Index<usize> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output
    {
        &self.0[index]
    }
}


impl<const D: usize> IndexMut<usize> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    fn index_mut(&mut self, index: usize) -> &mut f32
    {
        &mut self.0[index]
    }
}


impl<const D: usize> AsRef<[f32; D]> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    fn as_ref(&self) -> &[f32; D]
    {
        self.0.as_ref()
    }
}


impl<const D: usize> AsRef<[f32]> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    fn as_ref(&self) -> &[f32]
    {
        self.0.as_ref()
    }
}


impl<const D: usize> AsMut<[f32; D]> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    fn as_mut(&mut self) -> &mut [f32; D]
    {
        self.0.as_mut()
    }
}


impl<const D: usize> AsMut<[f32]> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    fn as_mut(&mut self) -> &mut [f32]
    {
        self.0.as_mut()
    }
}


impl<const D: usize> From<[f32; D]> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn from(value: [f32; D]) -> Self
    {
        Self(value.into())
    }
}


impl<const D: usize> Add<f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: f32) -> Self::Output
    {
        Self(self.0 + Simd::splat(rhs))
    }
}


impl<const D: usize> Add<Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: Vector<D>) -> Self::Output
    {
        Self(self.0 + rhs.0)
    }
}


impl<'rhs, const D: usize> Add<&'rhs f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: &'rhs f32) -> Self::Output
    {
        Self(self.0 + Simd::splat(*rhs))
    }
}


impl<'rhs, const D: usize> Add<&'rhs Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: &'rhs Vector<D>) -> Self::Output
    {
        Self(self.0 + rhs.0)
    }
}


impl<const D: usize> Sub<f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: f32) -> Self::Output
    {
        Self(self.0 - Simd::splat(rhs))
    }
}


impl<const D: usize> Sub<Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Vector<D>) -> Self::Output
    {
        Self(self.0 - rhs.0)
    }
}


impl<'rhs, const D: usize> Sub<&'rhs f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &'rhs f32) -> Self::Output
    {
        Self(self.0 - Simd::splat(*rhs))
    }
}


impl<'rhs, const D: usize> Sub<&'rhs Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &'rhs Vector<D>) -> Self::Output
    {
        Self(self.0 - rhs.0)
    }
}


impl<const D: usize> Mul<f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output
    {
        Self(self.0 * Simd::splat(rhs))
    }
}


impl<const D: usize> Mul<Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Vector<D>) -> Self::Output
    {
        Self(self.0 * rhs.0)
    }
}


impl<'rhs, const D: usize> Mul<&'rhs f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &'rhs f32) -> Self::Output
    {
        Self(self.0 * Simd::splat(*rhs))
    }
}


impl<'rhs, const D: usize> Mul<&'rhs Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &'rhs Vector<D>) -> Self::Output
    {
        Self(self.0 * rhs.0)
    }
}


impl<const D: usize> Div<f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output
    {
        Self(self.0 / Simd::splat(rhs))
    }
}


impl<const D: usize> Div<Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: Vector<D>) -> Self::Output
    {
        Self(self.0 / rhs.0)
    }
}


impl<'rhs, const D: usize> Div<&'rhs f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: &'rhs f32) -> Self::Output
    {
        Self(self.0 / Simd::splat(*rhs))
    }
}


impl<'rhs, const D: usize> Div<&'rhs Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: &'rhs Vector<D>) -> Self::Output
    {
        Self(self.0 / rhs.0)
    }
}


impl<const D: usize> AddAssign<f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn add_assign(&mut self, rhs: f32)
    {
        self.0 += Simd::splat(rhs)
    }
}


impl<const D: usize> AddAssign<Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn add_assign(&mut self, rhs: Vector<D>)
    {
        self.0 += rhs.0
    }
}


impl<'rhs, const D: usize> AddAssign<&'rhs f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn add_assign(&mut self, rhs: &'rhs f32)
    {
        self.0 += Simd::splat(*rhs)
    }
}


impl<'rhs, const D: usize> AddAssign<&'rhs Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn add_assign(&mut self, rhs: &'rhs Vector<D>)
    {
        self.0 += rhs.0
    }
}


impl<const D: usize> SubAssign<f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn sub_assign(&mut self, rhs: f32)
    {
        self.0 -= Simd::splat(rhs)
    }
}


impl<const D: usize> SubAssign<Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Vector<D>)
    {
        self.0 -= rhs.0
    }
}


impl<'rhs, const D: usize> SubAssign<&'rhs f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn sub_assign(&mut self, rhs: &'rhs f32)
    {
        self.0 -= Simd::splat(*rhs)
    }
}


impl<'rhs, const D: usize> SubAssign<&'rhs Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn sub_assign(&mut self, rhs: &'rhs Vector<D>)
    {
        self.0 -= rhs.0
    }
}


impl<const D: usize> MulAssign<f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn mul_assign(&mut self, rhs: f32)
    {
        self.0 *= Simd::splat(rhs)
    }
}


impl<const D: usize> MulAssign<Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn mul_assign(&mut self, rhs: Vector<D>)
    {
        self.0 *= rhs.0
    }
}


impl<'rhs, const D: usize> MulAssign<&'rhs f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn mul_assign(&mut self, rhs: &'rhs f32)
    {
        self.0 *= Simd::splat(*rhs)
    }
}


impl<'rhs, const D: usize> MulAssign<&'rhs Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn mul_assign(&mut self, rhs: &'rhs Vector<D>)
    {
        self.0 *= rhs.0
    }
}


impl<const D: usize> DivAssign<f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn div_assign(&mut self, rhs: f32)
    {
        self.0 /= Simd::splat(rhs)
    }
}


impl<const D: usize> DivAssign<Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn div_assign(&mut self, rhs: Vector<D>)
    {
        self.0 /= rhs.0
    }
}


impl<'rhs, const D: usize> DivAssign<&'rhs f32> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn div_assign(&mut self, rhs: &'rhs f32)
    {
        self.0 /= Simd::splat(*rhs)
    }
}


impl<'rhs, const D: usize> DivAssign<&'rhs Vector<D>> for Vector<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    #[inline]
    fn div_assign(&mut self, rhs: &'rhs Vector<D>)
    {
        self.0 /= rhs.0
    }
}
