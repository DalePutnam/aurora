use lua;
use na;
use primitives::Cube;
use primitives::Mesh;
use primitives::Sphere;
use rlua::Context;
use rlua::FromLua;
use rlua::Value;
use CookTorrance;
use Light;
use Material;
use Phong;

use super::SceneNode;

impl Phong
{
	pub fn lua_new<'lua>(
		lua: Context<'lua>,
		lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
	) -> rlua::Result<lua::Pointer<Material>>
	{
		let (lua_diffuse, lua_specular, lua_shininess) = lua_value;

		let diffuse = lua::Vector3::from_lua(lua_diffuse, lua)?;
		let specular = lua::Vector3::from_lua(lua_specular, lua)?;
		let shininess = f32::from_lua(lua_shininess, lua)?;

		Ok(lua::Pointer::new(Material::new(Phong::new(
			na::Vector3::from(diffuse),
			na::Vector3::from(specular),
			shininess,
		))))
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
	) -> rlua::Result<lua::Pointer<Material>>
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

		Ok(lua::Pointer::new(Material::new(CookTorrance::new(
			na::Vector3::from(diffuse_colour),
			na::Vector3::from(specular_colour),
			diffuse_fraction,
			roughness,
			refractive_index,
			extinction_coefficient,
		))))
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

		let mut node = lua::SceneNode::new(&name);
		let nh_sphere = lua::Pointer::new(Sphere::new(position.into(), radius));

		node.set_primitive(nh_sphere);

		Ok(node)
	}

	pub fn lua_unit_sphere<'lua>(
		lua: Context<'lua>,
		lua_name: Value<'lua>,
	) -> rlua::Result<lua::SceneNode>
	{
		let name = String::from_lua(lua_name, lua)?;

		let mut node = lua::SceneNode::new(&name);
		let sphere = lua::Pointer::new(Sphere::unit_sphere());

		node.set_primitive(sphere);

		Ok(node)
	}
}

impl Cube
{
	pub fn lua_unit_cube<'lua>(lua: Context<'lua>, lua_name: Value<'lua>)
		-> rlua::Result<lua::SceneNode>
	{
		let name = String::from_lua(lua_name, lua)?;

		let mut node = lua::SceneNode::new(&name);
		let sphere = lua::Pointer::new(Cube::unit_cube());

		node.set_primitive(sphere);

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

		let mut node = lua::SceneNode::new(&name);
		let nh_box = lua::Pointer::new(Cube::new(position.into(), size));

		node.set_primitive(nh_box);

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

		let mut node = lua::SceneNode::new(&name);
		let mesh = lua::Pointer::new(Mesh::new(&file_name));

		node.set_primitive(mesh);

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
		Ok(SceneNode::new(&name))
	}
}
