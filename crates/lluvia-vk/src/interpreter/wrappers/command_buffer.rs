use crate::command_buffer::{CommandBuffer, CommandBufferBuilder};
use crate::interpreter::wrappers::buffer::LuaBuffer;
use crate::interpreter::wrappers::image::LuaImage;
use std::cell::RefCell;

pub struct LuaOwnedCommandBufferBuilder(pub RefCell<Option<CommandBufferBuilder>>);

unsafe impl Send for LuaOwnedCommandBufferBuilder {}
unsafe impl Sync for LuaOwnedCommandBufferBuilder {}

impl mlua::UserData for LuaOwnedCommandBufferBuilder {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "copy_buffer_to_image",
            |_, this, (src, dst): (mlua::UserDataRef<LuaBuffer>, mlua::UserDataRef<LuaImage>)| {
                let mut opt = this.0.borrow_mut();
                let builder = opt.as_mut().ok_or_else(|| {
                    mlua::Error::RuntimeError("Command buffer builder already built/consumed".to_string())
                })?;
                builder
                    .copy_buffer_to_image(src.0.clone(), dst.0.clone())
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                Ok(())
            },
        );

        methods.add_method("build", |_, this, ()| {
            let mut opt = this.0.borrow_mut();
            let builder = opt.take().ok_or_else(|| {
                mlua::Error::RuntimeError("Command buffer builder already built/consumed".to_string())
            })?;
            let cmd_buf = builder
                .build_command_buffer()
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            Ok(LuaCommandBuffer(cmd_buf))
        });
    }
}

#[derive(Clone)]
pub struct LuaCommandBuffer(pub CommandBuffer);

unsafe impl Send for LuaCommandBuffer {}
unsafe impl Sync for LuaCommandBuffer {}

impl mlua::UserData for LuaCommandBuffer {}
