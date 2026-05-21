use crate::node::ComputeNodeDescriptor;

impl mlua::UserData for ComputeNodeDescriptor {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("function_name", |_a, this| Ok(this.function_name.clone()));

        fields.add_field_method_get("local_shape", |_a, this| Ok(this.local_shape));
    }
}
