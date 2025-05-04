mod mat;
mod mat_aliases;
mod quat;
mod vec_aliases;
mod vector;


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
