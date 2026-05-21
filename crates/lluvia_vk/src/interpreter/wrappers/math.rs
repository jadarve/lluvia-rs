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
}
