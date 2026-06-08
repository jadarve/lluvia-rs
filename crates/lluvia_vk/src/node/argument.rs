use crate::math;

#[derive(Clone, Copy, Debug)]
pub enum Argument {
    F32(f32),
    I32(i32),
    Bool(bool),
    Vec3(math::Vec3),
    UVec3(math::UVec3),
    UVec2(math::UVec2),
}

impl From<f32> for Argument {
    fn from(value: f32) -> Self {
        Argument::F32(value)
    }
}

impl From<i32> for Argument {
    fn from(value: i32) -> Self {
        Argument::I32(value)
    }
}

impl From<bool> for Argument {
    fn from(value: bool) -> Self {
        Argument::Bool(value)
    }
}

impl From<math::Vec3> for Argument {
    fn from(value: math::Vec3) -> Self {
        Argument::Vec3(value)
    }
}

impl From<math::UVec3> for Argument {
    fn from(value: math::UVec3) -> Self {
        Argument::UVec3(value)
    }
}

impl From<math::UVec2> for Argument {
    fn from(value: math::UVec2) -> Self {
        Argument::UVec2(value)
    }
}
