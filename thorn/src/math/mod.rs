mod mat;
mod mat_aliases;
mod quat;
mod vec_aliases;
mod vector;


use std::ops::Range;

pub use mat::*;
pub use mat_aliases::*;
pub use named_indices::*;
pub use quat::Quat;
pub use vec_aliases::*;
pub use vector::*;


pub mod named_indices
{
    pub const X: usize = 0;
    pub const Y: usize = 1;
    pub const Z: usize = 2;
    pub const W: usize = 3;
    pub const R: usize = 0;
    pub const G: usize = 1;
    pub const B: usize = 2;
    pub const A: usize = 3;
}


// Random. i'll use rand for now and wrap it in a stub interface
// in case i want to change something about the randomnes later
use rand::{Rng, distr::uniform::SampleUniform, seq::SliceRandom};


pub fn shuffle<T>(slice: &mut [T])
{
    slice.shuffle(&mut rand::rng());
}


pub fn random<T: PartialOrd + SampleUniform>(range: Range<T>) -> T
{
    rand::rng().random_range(range)
}
