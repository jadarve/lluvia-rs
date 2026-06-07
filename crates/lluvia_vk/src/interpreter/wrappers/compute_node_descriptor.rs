use crate::node::{ComputeNodeDescriptor, PortDescriptor};
use mlua::FromLua;

impl mlua::UserData for ComputeNodeDescriptor {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("function_name", |_, this| Ok(this.function_name.clone()));
        fields.add_field_method_set("function_name", |_, this, val: String| {
            this.function_name = val;
            Ok(())
        });

        fields.add_field_method_get("global_shape", |_, this| Ok(this.global_shape));
        fields.add_field_method_set("global_shape", |_, this, val: mlua::UserDataRef<crate::math::UVec3>| {
            this.global_shape = *val;
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

        methods.add_method("builder", |_, _this, ()| Ok(LuaComputeNodeDescriptorBuilder::default()));

        methods.add_method_mut("add_port", |_, this, port: mlua::UserDataRef<PortDescriptor>| {
            *this = this.clone().add_port(port.clone());
            Ok(())
        });

        methods.add_method_mut("set_constant", |lua, this, (name, value): (String, mlua::Value)| {
            let constant = crate::node::Constant::from_lua(value, lua)?;
            this.set_constant(name, constant);
            Ok(())
        });

        methods.add_method("get_constant", |_, this, name: String| {
            let constant = this
                .get_constant(&name)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            Ok(constant.clone())
        });
    }
}

#[derive(Clone, Default)]
pub struct LuaComputeNodeDescriptorBuilder {
    inner: ComputeNodeDescriptor,
}

impl mlua::UserData for LuaComputeNodeDescriptorBuilder {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::Call, |_, _self: mlua::Value| {
            Ok(LuaComputeNodeDescriptorBuilder::default())
        });

        methods.add_method_mut("function_name", |_, this, val: String| {
            this.inner.function_name = val;
            Ok(this.clone())
        });

        methods.add_method_mut("global_shape", |_, this, val: mlua::UserDataRef<crate::math::UVec3>| {
            this.inner.global_shape = *val;
            Ok(this.clone())
        });

        methods.add_method_mut(
            "program",
            |_, this, val: Option<mlua::UserDataRef<crate::program::Program>>| {
                this.inner.program = val.map(|p| p.clone());
                Ok(this.clone())
            },
        );

        methods.add_method_mut("add_port", |_, this, port: mlua::UserDataRef<PortDescriptor>| {
            this.inner = this.inner.clone().add_port(port.clone());
            Ok(this.clone())
        });

        methods.add_method_mut("add_constant", |lua, this, (name, value): (String, mlua::Value)| {
            let constant = crate::node::Constant::from_lua(value, lua)?;
            this.inner.set_constant(name, constant);
            Ok(this.clone())
        });

        methods.add_method("build", |_, this, ()| Ok(this.inner.clone()));
    }
}
