use std::ops::Deref;

use na;
use rlua::Context;
use rlua::Error;
use rlua::FromLua;
use rlua::Result;
use rlua::Value;

pub struct Vector3<T>(na::Vector3<T>)
where
	T: Copy + na::Scalar;

impl<T> Vector3<T>
where
	T: Copy + na::Scalar,
{
	pub fn new(x: T, y: T, z: T) -> Self
	{
		Vector3(na::Vector3::new(x, y, z))
	}
}

impl<T> Deref for Vector3<T>
where
	T: Copy + na::Scalar,
{
	type Target = na::Vector3<T>;

	fn deref(&self) -> &Self::Target
	{
		&self.0
	}
}

impl<T> From<Vector3<T>> for na::Vector3<T>
where
	T: Copy + na::Scalar,
{
	fn from(lua_vector: Vector3<T>) -> Self
	{
		lua_vector.0
	}
}

impl<'lua, T> FromLua<'lua> for Vector3<T>
where
	T: Copy + na::Scalar + FromLua<'lua> + 'static,
{
	fn from_lua(lua_value: Value<'lua>, _lua: Context<'lua>) -> Result<Self>
	{
		match lua_value {
			Value::Table(table) => {
				let table_length = table.len()?;

				if table_length != 3 {
					let msg = format!(
						"Expected a table of length 3, found length {}",
						table_length
					);
					Err(Error::FromLuaConversionError {
						from: "table",
						to: "Vector3",
						message: Some(msg),
					})
				} else {
					let x: T = table.get(1)?;
					let y: T = table.get(2)?;
					let z: T = table.get(3)?;

					Ok(Vector3::new(x, y, z))
				}
			},
			_ => {
				let msg = format!("Expected a table, found something else");
				Err(Error::FromLuaConversionError {
					from: "table",
					to: "Vector3",
					message: Some(msg),
				})
			},
		}
	}
}
