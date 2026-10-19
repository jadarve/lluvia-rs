pub struct LuaVectorUint8(pub Vec<u8>);

impl mlua::UserData for LuaVectorUint8 {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("size", |_, this, ()| Ok(this.0.len()));
    }
}
