use crate::node::{ComputeNodeDescriptor, PortDescriptor};

impl mlua::UserData for ComputeNodeDescriptor {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("function_name", |_, this| Ok(this.function_name.clone()));
        fields.add_field_method_set("function_name", |_, this, val: String| {
            this.function_name = val;
            Ok(())
        });

        fields.add_field_method_get("local_shape", |_, this| Ok(this.local_shape));
        fields.add_field_method_set("local_shape", |_, this, val: mlua::UserDataRef<crate::math::UVec3>| {
            this.local_shape = *val;
            Ok(())
        });

        fields.add_field_method_get("grid_shape", |_, this| Ok(this.grid_shape));
        fields.add_field_method_set("grid_shape", |_, this, val: mlua::UserDataRef<crate::math::UVec3>| {
            this.grid_shape = *val;
            Ok(())
        });

        fields.add_field_method_get("program", |_, this| Ok(this.program.clone()));
        fields.add_field_method_set(
            "program",
            |_, this, val: Option<mlua::UserDataRef<crate::program::Program>>| {
                this.program = val.map(|p| p.clone());
                Ok(())
            },
        );
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::Call, |_, _self: mlua::Value| {
            Ok(ComputeNodeDescriptor::default())
        });

        methods.add_method_mut("addPort", |_, this, port: mlua::UserDataRef<PortDescriptor>| {
            *this = this.clone().add_port(port.clone());
            Ok(())
        });

        methods.add_method_mut("add_port", |_, this, port: mlua::UserDataRef<PortDescriptor>| {
            *this = this.clone().add_port(port.clone());
            Ok(())
        });
    }
}
