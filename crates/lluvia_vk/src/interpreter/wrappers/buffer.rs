use crate::buffer::Buffer;
use crate::interpreter::wrappers::vector_uint8::LuaVectorUint8;
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

        methods.add_method(
            "map_and_set_from_vector_uint8",
            |_, this, data: mlua::UserDataRef<LuaVectorUint8>| {
                this.0.write(&data.0);
                Ok(())
            },
        );
    }
}
