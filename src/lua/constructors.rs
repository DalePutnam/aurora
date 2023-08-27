use std::sync::Arc;

use lua;
use na;
use primitives::Cube;
use primitives::Mesh;
use primitives::Sphere;
use rlua::Context;
use rlua::FromLua;
use rlua::Value;
use shading::Lambertian;
use Light;

use super::SceneNode;

impl Lambertian
{
	pub fn lua_new<'lua>(
		lua: Context<'lua>,
		lua_value: Value<'lua>,
	) -> rlua::Result<lua::Material>
	{
		let colour = lua::Vector3::from_lua(lua_value, lua)?;
		Ok(lua::Material::new(Lambertian::new(na::Vector3::from(colour))))
	}
}

impl Sphere
{
	pub fn lua_new<'lua>(
		lua: Context<'lua>,
		lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
	) -> rlua::Result<lua::SceneNode>
	{
		let (lua_name, lua_position, lua_radius) = lua_value;

		let name = String::from_lua(lua_name, lua)?;
		let position = lua::Vector3::from_lua(lua_position, lua)?;
		let radius = f32::from_lua(lua_radius, lua)?;

		let node = lua::SceneNode::new(&name, Some(Arc::new(Sphere::new(position.into(), radius))));

		Ok(node)
	}

	pub fn lua_unit_sphere<'lua>(
		lua: Context<'lua>,
		lua_name: Value<'lua>,
	) -> rlua::Result<lua::SceneNode>
	{
		let name = String::from_lua(lua_name, lua)?;

		let node = lua::SceneNode::new(&name, Some(Arc::new(Sphere::unit_sphere())));

		Ok(node)
	}
}

impl Cube
{
	pub fn lua_unit_cube<'lua>(
		lua: Context<'lua>,
		lua_name: Value<'lua>,
	) -> rlua::Result<lua::SceneNode>
	{
		let name = String::from_lua(lua_name, lua)?;

		let node = lua::SceneNode::new(&name, Some(Arc::new(Cube::unit_cube())));

		Ok(node)
	}

	pub fn lua_new<'lua>(
		lua: Context<'lua>,
		lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
	) -> rlua::Result<lua::SceneNode>
	{
		let (lua_name, lua_position, lua_size) = lua_value;

		let name = String::from_lua(lua_name, lua)?;
		let position = lua::Vector3::from_lua(lua_position, lua)?;
		let size = f32::from_lua(lua_size, lua)?;

		let node = lua::SceneNode::new(&name, Some(Arc::new(Cube::new(position.into(), size))));

		Ok(node)
	}
}

impl Mesh
{
	pub fn lua_new<'lua>(
		lua: Context<'lua>,
		lua_value: (Value<'lua>, Value<'lua>),
	) -> rlua::Result<lua::SceneNode>
	{
		let (lua_name, lua_file_name) = lua_value;

		let name = String::from_lua(lua_name, lua)?;
		let file_name = String::from_lua(lua_file_name, lua)?;

		let mesh = match Mesh::from_file(&file_name) {
			Ok(mesh) => mesh,
			Err(read_error) => return Err(rlua::Error::ExternalError(Arc::new(read_error))),
		};

		let node = lua::SceneNode::new(&name, Some(Arc::new(mesh)));

		Ok(node)
	}
}

impl Light
{
	pub fn lua_new<'lua>(
		lua: Context<'lua>,
		lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
	) -> rlua::Result<Light>
	{
		let (lua_position, lua_colour, lua_falloff) = lua_value;

		let position = lua::Vector3::from_lua(lua_position, lua)?;
		let colour = lua::Vector3::from_lua(lua_colour, lua)?;
		let falloff = lua::Vector3::from_lua(lua_falloff, lua)?;

		Ok(Light::new(
			na::Vector3::from(position),
			na::Vector3::from(colour),
			na::Vector3::from(falloff),
		))
	}

	pub fn lua_new_point_light<'lua>(
		lua: Context<'lua>,
		lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
	) -> rlua::Result<Light>
	{
		let (lua_position, lua_colour, lua_power) = lua_value;

		let position = lua::Vector3::from_lua(lua_position, lua)?;
		let colour = lua::Vector3::from_lua(lua_colour, lua)?;
		let power =  f32::from_lua(lua_power, lua)?;

		Ok(Light::new2(
			na::Vector3::from(position),
			na::Vector3::from(colour),
			power,
		))
	}
}

impl SceneNode
{
	pub fn lua_new<'lua>(lua: Context<'lua>, lua_name: Value<'lua>) -> rlua::Result<SceneNode>
	{
		let name = String::from_lua(lua_name, lua)?;
		Ok(SceneNode::new(&name, None))
	}
}
