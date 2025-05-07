#![no_std]

use spirv_std::glam::Vec4;
use spirv_std::spirv;


#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4)
{
    *output = Vec4::splat(1.0);
}
