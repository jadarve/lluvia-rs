//! Constant dynamic type wrapper.

/// Dynamic type wrapper for constants passed to nodes.
///
/// Mirrors C++ `ll::Parameter` renamed to `Constant`.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// 32-bit signed integer constant.
    Int(i32),
    /// 32-bit floating point constant.
    Float(f32),
    /// String constant.
    String(String),
}

impl mlua::FromLua for Constant {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::Integer(i) => Ok(Constant::Int(i as i32)),
            mlua::Value::Number(n) => {
                if n.fract() == 0.0 {
                    Ok(Constant::Int(n as i32))
                } else {
                    Ok(Constant::Float(n as f32))
                }
            }
            mlua::Value::String(s) => Ok(Constant::String(s.to_str()?.to_string())),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Constant".to_string(),
                message: Some("Expected Integer, Number, or String".to_string()),
            }),
        }
    }
}

impl mlua::IntoLua for Constant {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        match self {
            Constant::Int(i) => Ok(mlua::Value::Integer(i as i64)),
            Constant::Float(f) => Ok(mlua::Value::Number(f as f64)),
            Constant::String(s) => Ok(mlua::Value::String(lua.create_string(&s)?)),
        }
    }
}
