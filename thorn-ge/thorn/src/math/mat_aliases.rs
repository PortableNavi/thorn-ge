use super::{Matrix, Quat, Vec2, Vec2Ref, Vec3, Vec3Ref, Vec4, Vec4Ref, named_indices::*};

pub type Mat2 = Matrix<2>;
pub type Mat3 = Matrix<3>;
pub type Mat4 = Matrix<4>;


impl Mat2
{
    pub fn of(x: Vec2, y: Vec2) -> Self
    {
        Self::from([x, y])
    }

    pub fn ovl(&self) -> Mat2Ref
    {
        Mat2Ref::from(self)
    }
}


impl std::fmt::Debug for Mat2
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Mat2[{:?}, {:?}]", self[X], self[Y])
    }
}


impl std::fmt::Display for Mat2
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{self:?}")
    }
}


impl Mat3
{
    pub fn of(x: Vec3, y: Vec3, z: Vec3) -> Self
    {
        Self::from([x, y, z])
    }

    pub fn ovl(&self) -> Mat3Ref
    {
        Mat3Ref::from(self)
    }
}


impl std::fmt::Debug for Mat3
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Mat3[{:?}, {:?}, {:?}]", self[X], self[Y], self[Z])
    }
}


impl std::fmt::Display for Mat3
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{self:?}")
    }
}


impl Mat4
{
    pub fn of(x: Vec4, y: Vec4, z: Vec4, w: Vec4) -> Self
    {
        Self::from([x, y, z, w])
    }

    pub fn ovl(&self) -> Mat4Ref
    {
        Mat4Ref::from(self)
    }

    pub fn orthographic(left_down: Vec2, extent: Vec2, clipping: Vec2) -> Self
    {
        let mut out = Mat4::new();

        let start = left_down;
        let end = left_down + extent;

        let xy = Vec2::splat(1.0) / (end - start);
        let clip = (-1.0) / (clipping[X] - clipping[Y]);

        out[0][0] = 2.0 * xy[X];
        out[1][1] = 2.0 * xy[Y];
        out[2][2] = 2.0 * clip;

        out[3][0] = -(start[X] + end[X]) * xy[X];
        out[3][1] = -(start[Y] + end[Y]) * xy[Y];
        out[3][2] = clipping.sum() * clip;

        out
    }

    pub fn perspective(fov_deg: f32, extent: Vec2, clipping: Vec2) -> Self
    {
        let fov = fov_deg.to_radians();
        let fov_ht = (fov * 0.5).tan();
        let aspect = extent[X] / extent[Y];

        let mut ret = Mat4::new();

        ret[0][0] = 1.0 / (aspect * fov_ht);
        ret[1][1] = 1.0 / fov_ht;
        ret[2][2] = clipping.sum() / (clipping[X] - clipping[Y]);
        ret[2][3] = -1.0;
        ret[3][2] = (2.0 * clipping[X] * clipping[Y]) / (clipping[X] - clipping[Y]);

        ret
    }

    pub fn lookat(pos: Vec3, target: Vec3, top: Vec3) -> Self
    {
        let mut ret = Mat4::new();

        let z_axis = (target - pos).norm();
        let x_axis = z_axis.cross(top).norm();
        let y_axis = x_axis.cross(z_axis);

        ret[0] = Vec4::new(x_axis[X], y_axis[X], -z_axis[X], 0.0);
        ret[1] = Vec4::new(x_axis[Y], y_axis[Y], -z_axis[Y], 0.0);
        ret[2] = Vec4::new(x_axis[Z], y_axis[Z], -z_axis[Z], 0.0);
        ret[4] = Vec4::new(-x_axis.dot(pos), -y_axis.dot(pos), z_axis.dot(pos), 1.0);

        ret
    }

    pub fn invert_unchecked(&self) -> Mat4
    {
        //God save me !!!
        todo!()
    }

    pub fn translation(&self) -> Vec3
    {
        Vec3::new(self[W][X], self[W][Y], self[W][Z])
    }

    pub fn set_translation(&mut self, t: Vec3)
    {
        self[W][X] = t[X];
        self[W][Y] = t[Y];
        self[W][Z] = t[Z];
    }

    pub fn from_translation(pos: Vec3) -> Self
    {
        let mut ret = Mat4::new();
        ret.set_translation(pos);
        ret
    }

    pub fn scale(&self) -> Vec3
    {
        Vec3::new(self[0][0], self[1][1], self[2][2])
    }

    pub fn set_scale(&mut self, s: Vec3)
    {
        self[0][0] = s[X];
        self[1][1] = s[Y];
        self[2][2] = s[Z];
    }

    pub fn from_scale(scale: Vec3) -> Self
    {
        let mut ret = Mat4::new();
        ret.set_scale(scale);
        ret
    }

    pub fn from_euler_x(rad: f32) -> Self
    {
        let mut ret = Mat4::new();
        let cos = rad.cos();
        let sin = rad.sin();

        ret[1][1] = cos;
        ret[1][2] = sin;
        ret[2][1] = -sin;
        ret[2][2] = sin;

        ret
    }

    pub fn from_euler_y(rad: f32) -> Self
    {
        let mut ret = Mat4::new();
        let cos = rad.cos();
        let sin = rad.sin();

        ret[0][0] = cos;
        ret[0][2] = -sin;
        ret[2][0] = sin;
        ret[2][2] = cos;

        ret
    }

    pub fn from_euler_z(rad: f32) -> Self
    {
        let mut ret = Mat4::new();
        let cos = rad.cos();
        let sin = rad.sin();

        ret[0][0] = cos;
        ret[0][1] = sin;
        ret[1][0] = -sin;
        ret[1][1] = cos;

        ret
    }

    pub fn from_euler(xyz_rad: Vec3) -> Self
    {
        let x = Self::from_euler_x(xyz_rad[X]);
        let y = Self::from_euler_y(xyz_rad[Y]);
        let z = Self::from_euler_z(xyz_rad[Z]);

        (x * y) * z
    }

    pub fn forward(&self) -> Vec3
    {
        Vec3::new(-self[0][2], -self[1][2], -self[2][2]).norm()
    }

    pub fn backwards(&self) -> Vec3
    {
        self.forward() * -1.0
    }

    pub fn up(&self) -> Vec3
    {
        Vec3::new(-self[0][1], -self[1][1], -self[2][1]).norm()
    }

    pub fn down(&self) -> Vec3
    {
        self.up() * -1.0
    }

    pub fn left(&self) -> Vec3
    {
        Vec3::new(-self[0][0], -self[1][0], -self[2][0]).norm()
    }

    pub fn right(&self) -> Vec3
    {
        self.forward() * -1.0
    }
}


impl std::fmt::Debug for Mat4
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "Mat4[{:?}, {:?}, {:?}, {:?}]",
            self[X], self[Y], self[Z], self[W]
        )
    }
}


impl std::fmt::Display for Mat4
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{self:?}")
    }
}


impl From<Quat> for Mat4
{
    fn from(value: Quat) -> Self
    {
        let n = value.norm();
        let mut ret = Mat4::new();

        ret[0][0] = 1.0 - 2.0 * n.j * n.j - 2.0 * n.k * n.k;
        ret[0][1] = 2.0 * n.i * n.j - 2.0 * n.k * n.r;
        ret[0][2] = 2.0 * n.i * n.j + 2.0 * n.j * n.r;

        ret[1][0] = 2.0 * n.i * n.j + 2.0 * n.k * n.r;
        ret[1][1] = 1.0 - 2.0 * n.i * n.i - 2.0 * n.k * n.k;
        ret[1][2] = 2.0 * n.j * n.k - 2.0 * n.i * n.r;

        ret[2][0] = 2.0 * n.i * n.k - 2.0 * n.j * n.r;
        ret[2][1] = 2.0 * n.j * n.k + 2.0 * n.i * n.r;
        ret[2][2] = 1.0 - 2.0 * n.i * n.i - 2.0 * n.j * n.j;

        ret
    }
}


pub struct Mat2Ref<'a>
{
    pub x: Vec2Ref<'a>,
    pub y: Vec2Ref<'a>,
}


impl Mat2Ref<'_>
{
    pub fn mat2(&self) -> Mat2
    {
        Mat2::from([self.x.vec2(), self.y.vec2()])
    }
}


impl<'a> From<&'a Mat2> for Mat2Ref<'a>
{
    fn from(value: &'a Mat2) -> Self
    {
        Self {
            x: value[X].ovl(),
            y: value[Y].ovl(),
        }
    }
}


pub struct Mat3Ref<'a>
{
    pub x: Vec3Ref<'a>,
    pub y: Vec3Ref<'a>,
    pub z: Vec3Ref<'a>,
}


impl Mat3Ref<'_>
{
    pub fn mat3(&self) -> Mat3
    {
        Mat3::from([self.x.vec3(), self.y.vec3(), self.z.vec3()])
    }
}


impl<'a> From<&'a Mat3> for Mat3Ref<'a>
{
    fn from(value: &'a Mat3) -> Self
    {
        Self {
            x: value[X].ovl(),
            y: value[Y].ovl(),
            z: value[Z].ovl(),
        }
    }
}


pub struct Mat4Ref<'a>
{
    pub x: Vec4Ref<'a>,
    pub y: Vec4Ref<'a>,
    pub z: Vec4Ref<'a>,
    pub w: Vec4Ref<'a>,
}


impl Mat4Ref<'_>
{
    pub fn mat4(&self) -> Mat4
    {
        Mat4::from([self.x.vec4(), self.y.vec4(), self.z.vec4(), self.w.vec4()])
    }
}


impl<'a> From<&'a Mat4> for Mat4Ref<'a>
{
    fn from(value: &'a Mat4) -> Self
    {
        Self {
            x: value[X].ovl(),
            y: value[Y].ovl(),
            z: value[Z].ovl(),
            w: value[W].ovl(),
        }
    }
}
