use crate::buffer::Buffer;
use std::sync::Arc;

#[derive(Clone)]
pub struct LuaBuffer(pub Arc<Buffer>);

impl mlua::UserData for LuaBuffer {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("size", |_, this, ()| Ok(this.0.size()));

        methods.add_meta_method(mlua::MetaMethod::Div, |_, this, divisor: u64| {
            Ok(this.0.size() / divisor)
        });

        methods.add_meta_method(mlua::MetaMethod::IDiv, |_, this, divisor: u64| {
            Ok(this.0.size() / divisor)
        });
    }
}
