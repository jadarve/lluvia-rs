use mlua;

use crate::SessionDescriptor;

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
// impl mlua::UserData for BufferDe
