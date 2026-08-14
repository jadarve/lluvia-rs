use crate::image::{Image, ImageDescriptor, ImageViewDescriptor};
use crate::interpreter::wrappers::image_view::LuaImageView;
use std::sync::Arc;

pub struct LuaImage(pub Arc<Image>);

impl mlua::UserData for LuaImage {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "create_image_view",
            |lua, this, view_desc: mlua::UserDataRef<ImageViewDescriptor>| {
                let view = this
                    .0
                    .create_image_view(&view_desc)
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                let ud = lua.create_userdata(LuaImageView(view))?;
                Ok(mlua::Value::UserData(ud))
            },
        );
    }
}

impl mlua::UserData for ImageDescriptor {}
impl mlua::UserData for ImageViewDescriptor {}
