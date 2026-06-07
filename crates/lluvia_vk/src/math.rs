/// Floating point 3D vector
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

    pub fn x(&self) -> f32 {
        self.inner.x
    }

    pub fn y(&self) -> f32 {
        self.inner.y
    }

    pub fn z(&self) -> f32 {
        self.inner.z
    }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
    }
}

impl From<&Vec3> for Vec3 {
    fn from(v: &Vec3) -> Self {
        *v
    }
}

/// Unsigned integer 3D vector
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

    pub fn x(&self) -> u32 {
        self.inner.x
    }

    pub fn y(&self) -> u32 {
        self.inner.y
    }

    pub fn z(&self) -> u32 {
        self.inner.z
    }
}

impl std::fmt::Display for UVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
    }
}

impl From<&UVec3> for UVec3 {
    fn from(v: &UVec3) -> Self {
        *v
    }
}

impl From<(u32, u32, u32)> for UVec3 {
    fn from(v: (u32, u32, u32)) -> Self {
        Self::new(v.0, v.1, v.2)
    }
}

impl From<&(u32, u32, u32)> for UVec3 {
    fn from(v: &(u32, u32, u32)) -> Self {
        Self::new(v.0, v.1, v.2)
    }
}
