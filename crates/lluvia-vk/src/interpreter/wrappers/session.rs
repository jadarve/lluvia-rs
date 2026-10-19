use mlua;

use crate::{Session, SessionDescriptor};

///////////////////////////////////////////////////////////////////////////////
impl mlua::UserData for SessionDescriptor {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("enable_debug", |_, desc| Ok(desc.enable_debug));
        fields.add_field_method_set("enable_debug", |_, desc, value| {
            desc.enable_debug = value;
            Ok(())
        });
    }
}

///////////////////////////////////////////////////////////////////////////////
// impl UserData for Session
impl mlua::UserData for Session {
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("load_program", |_lua, session, path: String| {
            match session.load_program(&path) {
                Ok(program) => Ok(program),
                Err(e) => Err(mlua::Error::RuntimeError(e.to_string())),
            }
        });
    }
}
