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
        fields.add_field_method_set("program", |_, this, val: mlua::UserDataRef<crate::program::Program>| {
            this.program = val.clone();
            Ok(())
        });
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
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
    ports: Vec<PortDescriptor>,
    constants: std::collections::HashMap<String, crate::node::Constant>,
    global_shape: Option<crate::math::UVec3>,
    program: Option<crate::program::Program>,
    function_name: Option<String>,
}

impl mlua::UserData for LuaComputeNodeDescriptorBuilder {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::Call, |_, _self: mlua::Value| {
            Ok(LuaComputeNodeDescriptorBuilder::default())
        });

        methods.add_method_mut("function_name", |_, this, val: String| {
            this.function_name = Some(val);
            Ok(this.clone())
        });

        methods.add_method_mut("global_shape", |_, this, val: mlua::UserDataRef<crate::math::UVec3>| {
            this.global_shape = Some(*val);
            Ok(this.clone())
        });

        methods.add_method_mut(
            "program",
            |_, this, val: Option<mlua::UserDataRef<crate::program::Program>>| {
                this.program = val.map(|p| p.clone());
                Ok(this.clone())
            },
        );

        methods.add_method_mut("add_port", |_, this, port: mlua::UserDataRef<PortDescriptor>| {
            this.ports.push(port.clone());
            Ok(this.clone())
        });

        methods.add_method_mut("add_constant", |lua, this, (name, value): (String, mlua::Value)| {
            let constant = crate::node::Constant::from_lua(value, lua)?;
            this.constants.insert(name, constant);
            Ok(this.clone())
        });

        methods.add_method("build", |_, this, ()| {
            let program = this
                .program
                .clone()
                .ok_or_else(|| mlua::Error::RuntimeError("program is required".to_string()))?;

            let mut builder = ComputeNodeDescriptor::builder()
                .global_shape(this.global_shape.unwrap_or(crate::math::UVec3::ONE))
                .program(program)
                .function_name(this.function_name.clone().unwrap_or_else(|| "main".to_string()));
            for port in this.ports.iter() {
                builder = builder.add_port(port.clone());
            }
            for (name, value) in this.constants.iter() {
                builder = builder.add_constant(name.clone(), value.clone());
            }
            Ok(builder.build())
        });
    }
}
