use crate::math;

///////////////////////////////////////////////////////////
impl mlua::UserData for math::Vec3 {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.inner.x));
        fields.add_field_method_get("y", |_, this| Ok(this.inner.y));
        fields.add_field_method_get("z", |_, this| Ok(this.inner.z));
        fields.add_field_method_set("x", |_, this, x| {
            this.inner.x = x;
            Ok(())
        });
        fields.add_field_method_set("y", |_, this, y| {
            this.inner.y = y;
            Ok(())
        });
        fields.add_field_method_set("z", |_, this, z| {
            this.inner.z = z;
            Ok(())
        });
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        // corresponds to Vec3() function in native.d.luau
        methods.add_meta_function(
            mlua::MetaMethod::Call,
            |_, (_self, x, y, z): (mlua::Value, f32, f32, f32)| Ok(math::Vec3::new(x, y, z)),
        );
    }
}

///////////////////////////////////////////////////////////
impl mlua::UserData for math::UVec3 {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.inner.x));
        fields.add_field_method_get("y", |_, this| Ok(this.inner.y));
        fields.add_field_method_get("z", |_, this| Ok(this.inner.z));
        fields.add_field_method_set("x", |_, this, x| {
            this.inner.x = x;
            Ok(())
        });
        fields.add_field_method_set("y", |_, this, y| {
            this.inner.y = y;
            Ok(())
        });
        fields.add_field_method_set("z", |_, this, z| {
            this.inner.z = z;
            Ok(())
        });
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        // corresponds to UVec3(x, y, z) function in native.d.luau
        methods.add_meta_function(
            mlua::MetaMethod::Call,
            |_, (_self, x, y, z): (mlua::Value, u32, u32, u32)| Ok(math::UVec3::new(x, y, z)),
        );
    }
}
