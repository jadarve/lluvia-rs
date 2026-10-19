use crate::node::PushConstants;

impl mlua::UserData for PushConstants {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::Call, |_, _self: mlua::Value| {
            Ok(PushConstants::default())
        });

        methods.add_method_mut("push_f32", |_, this, val: f32| {
            this.push_f32(val);
            Ok(())
        });

        methods.add_method_mut("push_i32", |_, this, val: i32| {
            this.push_i32(val);
            Ok(())
        });
    }
}
