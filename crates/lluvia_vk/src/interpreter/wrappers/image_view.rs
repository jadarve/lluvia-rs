use crate::image::ImageView;
use std::sync::Arc;

#[derive(Clone)]
pub struct LuaImageView(pub Arc<ImageView>);

impl mlua::UserData for LuaImageView {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| Ok(this.0.image().descriptor().width));
        fields.add_field_method_get("height", |_, this| Ok(this.0.image().descriptor().height));
        fields.add_field_method_get("depth", |_, this| Ok(this.0.image().descriptor().depth));
        fields.add_field_method_get("channel_count", |_, this| {
            Ok(this.0.image().descriptor().channel_count as u32)
        });
    }
}
