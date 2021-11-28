use std::sync::Arc;

use lua;
use na;
use primitives::Cube;
use primitives::Mesh;
use primitives::Sphere;
use rlua::Context;
use rlua::FromLua;
use rlua::Value;
use shading::CookTorrance;
use shading::Phong;
use Light;

use super::SceneNode;

impl Phong
{
	pub fn lua_new<'lua>(
		lua: Context<'lua>,
		lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
	) -> rlua::Result<lua::Material>
	{
		let (lua_diffuse, lua_specular, lua_shininess) = lua_value;

		let diffuse = lua::Vector3::from_lua(lua_diffuse, lua)?;
		let specular = lua::Vector3::from_lua(lua_specular, lua)?;
		let shininess = f32::from_lua(lua_shininess, lua)?;

		Ok(lua::Material::new(Phong::new(
			na::Vector3::from(diffuse),
			na::Vector3::from(specular),
			shininess,
		)))
	}
}

impl CookTorrance
{
	pub fn lua_new<'lua>(
		lua: Context<'lua>,
		lua_value: (
			Value<'lua>,
			Value<'lua>,
			Value<'lua>,
			Value<'lua>,
			Value<'lua>,
			Value<'lua>,
		),
	) -> rlua::Result<lua::Material>
	{
		let (
			lua_diffuse_colour,
			lua_specular_colour,
			lua_diffuse_fraction,
			lua_roughness,
			lua_refractive_index,
			lua_extinction_coefficient,
		) = lua_value;

		let diffuse_colour = lua::Vector3::from_lua(lua_diffuse_colour, lua)?;
		let specular_colour = lua::Vector3::from_lua(lua_specular_colour, lua)?;
		let diffuse_fraction = f32::from_lua(lua_diffuse_fraction, lua)?;
		let roughness = f32::from_lua(lua_roughness, lua)?;
		let refractive_index = f32::from_lua(lua_refractive_index, lua)?;
		let extinction_coefficient = f32::from_lua(lua_extinction_coefficient, lua)?;

		Ok(lua::Material::new(CookTorrance::new(
			na::Vector3::from(diffuse_colour),
			na::Vector3::from(specular_colour),
			diffuse_fraction,
			roughness,
			refractive_index,
			extinction_coefficient,
		)))
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

		let node =
			lua::SceneNode::new(&name, Some(Arc::new(Sphere::new(position.into(), radius))));

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

		let node = lua::SceneNode::new(&name, Some(Arc::new(Mesh::new(&file_name))));

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
}

impl SceneNode
{
	pub fn lua_new<'lua>(lua: Context<'lua>, lua_name: Value<'lua>) -> rlua::Result<SceneNode>
	{
		let name = String::from_lua(lua_name, lua)?;
		Ok(SceneNode::new(&name, None))
	}
}
