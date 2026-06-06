#[derive(Clone, Copy, Default, Debug)]
pub struct Vec3 {
    pub inner: glam::Vec3,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        inner: glam::Vec3::ZERO,
    };

    pub const ONE: Self = Self { inner: glam::Vec3::ONE };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: glam::Vec3::new(x, y, z),
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct UVec3 {
    pub inner: glam::UVec3,
}

impl UVec3 {
    pub const ZERO: Self = Self {
        inner: glam::UVec3::ZERO,
    };

    pub const ONE: Self = Self {
        inner: glam::UVec3::ONE,
    };
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self {
            inner: glam::UVec3::new(x, y, z),
        }
    }
}

impl From<&UVec3> for UVec3 {
    fn from(v: &UVec3) -> Self {
        *v
    }
}

impl From<&Vec3> for Vec3 {
    fn from(v: &Vec3) -> Self {
        *v
    }
}
