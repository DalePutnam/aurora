use std::ops::Deref;
use std::sync::Arc;

use rlua::UserData;
use rlua::UserDataMethods;

pub struct Pointer<T: ?Sized>(Arc<T>);

impl<T> Pointer<T>
{
	pub fn new(value: T) -> Pointer<T>
	{
		Pointer(Arc::new(value))
	}
}

impl<T: ?Sized> Deref for Pointer<T>
{
	type Target = Arc<T>;

	fn deref(&self) -> &Self::Target
	{
		&self.0
	}
}

impl<T: ?Sized> Clone for Pointer<T>
{
	fn clone(&self) -> Self
	{
		Pointer(self.0.clone())
	}
}

impl<T: ?Sized> From<Arc<T>> for Pointer<T>
{
	fn from(arc: Arc<T>) -> Self
	{
		Pointer(arc)
	}
}

impl<T: ?Sized> From<Pointer<T>> for Arc<T>
{
	fn from(lua_ptr: Pointer<T>) -> Self
	{
		lua_ptr.0
	}
}

impl<T: ?Sized> UserData for Pointer<T>
{
	fn add_methods<'lua, U: UserDataMethods<'lua, Self>>(_methods: &mut U) {}
}
