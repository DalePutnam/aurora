use rlua::UserData;
use rlua::UserDataMethods;
use Light;

impl UserData for Light
{
	fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(_methods: &mut T) {}
}
