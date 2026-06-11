use crate::node::{ContainerNodeDescriptor, PortDescriptor};
use mlua::FromLua;

impl mlua::UserData for ContainerNodeDescriptor {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("builder_name", |_, this| Ok(this.builder_name.clone()));
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_constant", |_, this, name: String| {
            let constant = this
                .get_constant(&name)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            Ok(constant.clone())
        });

        methods.add_method("getConstant", |_, this, name: String| {
            let constant = this
                .get_constant(&name)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            Ok(constant.clone())
        });
    }
}

#[derive(Clone, Default)]
pub struct LuaContainerNodeDescriptorBuilder {
    ports: Vec<PortDescriptor>,
    constants: std::collections::HashMap<String, crate::node::Constant>,
    builder_name: Option<String>,
}

impl mlua::UserData for LuaContainerNodeDescriptorBuilder {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::Call, |_, _self: mlua::Value| {
            Ok(LuaContainerNodeDescriptorBuilder::default())
        });

        methods.add_method_mut("builder_name", |_, this, val: String| {
            this.builder_name = Some(val);
            Ok(this.clone())
        });

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
            let builder_name = this.builder_name.clone().unwrap_or_default();
            let mut builder = ContainerNodeDescriptor::builder().builder_name(builder_name);
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
