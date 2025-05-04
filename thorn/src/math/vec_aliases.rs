use super::named_indices::*;
use super::vector::*;
use std::simd::Simd;


pub type Vec2 = Vector<2>;
pub type Vec3 = Vector<3>;
pub type Vec4 = Vector<4>;


pub struct Vec2Ref<'a>
{
    pub x: &'a f32,
    pub y: &'a f32,
}


impl<'a> Vec2Ref<'a>
{
    pub fn vec2(&self) -> Vec2
    {
        Vec2::new(*self.x, *self.y)
    }
}


impl<'a> From<&'a Vec2> for Vec2Ref<'a>
{
    #[inline]
    fn from(value: &'a Vec2) -> Self
    {
        Self {
            x: &value[X],
            y: &value[Y],
        }
    }
}


pub struct Vec3Ref<'a>
{
    pub x: &'a f32,
    pub y: &'a f32,
    pub z: &'a f32,
}


impl<'a> Vec3Ref<'a>
{
    pub fn vec3(&self) -> Vec3
    {
        Vec3::new(*self.x, *self.y, *self.z)
    }
}


impl<'a> From<&'a Vec3> for Vec3Ref<'a>
{
    #[inline]
    fn from(value: &'a Vec3) -> Self
    {
        Self {
            x: &value[X],
            y: &value[Y],
            z: &value[Z],
        }
    }
}


pub struct Vec4Ref<'a>
{
    pub x: &'a f32,
    pub y: &'a f32,
    pub z: &'a f32,
    pub w: &'a f32,
}


impl<'a> Vec4Ref<'a>
{
    pub fn vec4(&self) -> Vec4
    {
        Vec4::new(*self.x, *self.y, *self.z, *self.w)
    }
}


impl<'a> From<&'a Vec4> for Vec4Ref<'a>
{
    #[inline]
    fn from(value: &'a Vec4) -> Self
    {
        Self {
            x: &value[X],
            y: &value[Y],
            z: &value[Z],
            w: &value[W],
        }
    }
}


impl Vec2
{
    pub const UP: Self = Self(Simd::from_array([0.0, 1.0]));
    pub const DOWN: Self = Self(Simd::from_array([0.0, -1.0]));
    pub const LEFT: Self = Self(Simd::from_array([-1.0, 0.0]));
    pub const RIGHT: Self = Self(Simd::from_array([1.0, 0.0]));

    #[inline]
    pub fn new(x: f32, y: f32) -> Self
    {
        Self::from([x, y])
    }

    #[inline]
    pub fn ovl(&self) -> Vec2Ref
    {
        self.into()
    }
}


impl Vec3
{
    pub const UP: Self = Self(Simd::from_array([0.0, 1.0, 0.0]));
    pub const DOWN: Self = Self(Simd::from_array([0.0, -1.0, 0.0]));
    pub const LEFT: Self = Self(Simd::from_array([-1.0, 0.0, 0.0]));
    pub const RIGHT: Self = Self(Simd::from_array([1.0, 0.0, 0.0]));
    pub const FORWARD: Self = Self(Simd::from_array([0.0, 0.0, 1.0]));
    pub const BACKWARD: Self = Self(Simd::from_array([0.0, 0.0, -1.0]));

    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self
    {
        Self::from([x, y, z])
    }

    #[inline]
    pub fn ovl(&self) -> Vec3Ref
    {
        self.into()
    }

    #[inline]
    pub fn cross(&self, other: Vec3) -> Self
    {
        let a = self.ovl();
        let b = other.ovl();

        Self::new(
            a.y * b.z - a.z * b.y,
            a.z * b.x - a.x * b.z,
            a.x * b.y - a.y * b.x,
        )
    }
}


impl Vec4
{
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self
    {
        Self::from([x, y, z, w])
    }

    #[inline]
    pub fn ovl(&self) -> Vec4Ref
    {
        self.into()
    }
}


impl std::fmt::Debug for Vec2
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Vec2({}, {})", self[X], self[Y])
    }
}


impl std::fmt::Display for Vec2
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{self:?}")
    }
}


impl std::fmt::Debug for Vec3
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Vec3({}, {}, {})", self[X], self[Y], self[Z])
    }
}


impl std::fmt::Display for Vec3
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{self:?}")
    }
}


impl std::fmt::Debug for Vec4
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "Vec4({}, {}, {}, {})",
            self[X], self[Y], self[Z], self[W]
        )
    }
}


impl std::fmt::Display for Vec4
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{self:?}")
    }
}
