// Allow needles range loops for this module because i think
// that the implemetations of the matrix based operations implemented here
// are better expressed and easier to read using range loops.
#![allow(clippy::needless_range_loop)]


use super::vector::Vector;
use std::{
    ops::{Index, IndexMut, Mul},
    simd::{LaneCount, SupportedLaneCount},
};


#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct Matrix<const D: usize>([Vector<D>; D])
where
    LaneCount<D>: SupportedLaneCount;


impl<const D: usize> Matrix<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    pub fn new() -> Self
    {
        let mut axis = [Vector::ZERO; D];

        for i in 0..D
        {
            axis[i][i] = 1.0;
        }

        Self(axis)
    }

    pub fn zero() -> Self
    {
        Self([Vector::ZERO; D])
    }

    pub fn transpose(&self) -> Self
    {
        let mut axis = [Vector::ZERO; D];

        for n in 0..D
        {
            for m in 0..D
            {
                axis[m][n] = self[n][m];
            }
        }

        Self(axis)
    }
}


impl<const D: usize> From<[Vector<D>; D]> for Matrix<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    fn from(value: [Vector<D>; D]) -> Self
    {
        Self(value)
    }
}


impl<const D: usize> Default for Matrix<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    fn default() -> Self
    {
        Self::new()
    }
}


impl<const D: usize> Index<usize> for Matrix<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Vector<D>;

    fn index(&self, index: usize) -> &Self::Output
    {
        &self.0[index]
    }
}


impl<const D: usize> IndexMut<usize> for Matrix<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    fn index_mut(&mut self, index: usize) -> &mut Vector<D>
    {
        &mut self.0[index]
    }
}


impl<const D: usize> Mul<Self> for Matrix<D>
where
    LaneCount<D>: SupportedLaneCount,
{
    type Output = Self;

    fn mul(self, rhs: Matrix<D>) -> Self::Output
    {
        let rhs = rhs.transpose();

        let mut axis = [Vector::ZERO; D];

        for n in 0..D
        {
            for m in 0..D
            {
                axis[n][m] = self[n].dot(rhs[m]);
            }
        }

        Self(axis)
    }
}
