use crate::node::{PortDescriptor, PortDirection, PortType};

impl mlua::UserData for PortDescriptor {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(
            mlua::MetaMethod::Call,
            |_, (_self, binding, name, direction, port_type): (mlua::Value, u32, String, u32, u32)| {
                let direction = PortDirection::try_from(direction).map_err(mlua::Error::external)?;
                let port_type = PortType::try_from(port_type).map_err(mlua::Error::external)?;

                Ok(PortDescriptor {
                    binding,
                    name,
                    direction,
                    port_type,
                })
            },
        );
    }

    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("binding", |_, this| Ok(this.binding));
        fields.add_field_method_set("binding", |_, this, val: u32| {
            this.binding = val;
            Ok(())
        });

        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_set("name", |_, this, val: String| {
            this.name = val;
            Ok(())
        });

        fields.add_field_method_get("direction", |_, this| Ok(u32::from(this.direction)));
        fields.add_field_method_set("direction", |_, this, val: u32| {
            this.direction = PortDirection::try_from(val).map_err(mlua::Error::external)?;
            Ok(())
        });

        fields.add_field_method_get("port_type", |_, this| Ok(u32::from(this.port_type)));
        fields.add_field_method_set("port_type", |_, this, val: u32| {
            this.port_type = PortType::try_from(val).map_err(mlua::Error::external)?;
            Ok(())
        });
    }
}
