use Light;
use rlua::{UserData, UserDataMethods};

impl UserData for Light {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(_methods: &mut T) {}
}
