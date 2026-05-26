use crate::program::Program;

impl mlua::UserData for Program {
    // nothing to do. Program is an opaque type
}
