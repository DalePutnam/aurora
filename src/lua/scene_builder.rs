use std::fs::File;
use std::io::Read;

use cli;
use lua;
use primitives::Cube;
use primitives::Mesh;
use primitives::Sphere;
use render;
use rlua::Context;
use rlua::FromLua;
use rlua::Lua;
use rlua::Value;
use shading::CookTorrance;
use shading::Phong;
use Light;

pub struct SceneBuilder
{
	lua: Lua,
}

impl SceneBuilder
{
	pub fn new(parameters: cli::Parameters) -> Self
	{
		let mut lua = Lua::new();
		SceneBuilder::initialize_environment(&mut lua, parameters);

		SceneBuilder { lua: lua }
	}

	pub fn run_build_script(&self, script_path: &String) -> Result<(), String>
	{
		let mut file = match File::open(script_path) {
			Ok(f) => f,
			Err(e) => return Err(e.to_string()),
		};

		let mut contents = String::new();
		match file.read_to_string(&mut contents) {
			Ok(s) => s,
			Err(e) => return Err(e.to_string()),
		};

		let result = self.lua.context(|lua_ctx| -> rlua::Result<()> {
			lua_ctx.load(&contents).exec()?;
			Ok(())
		});

		match result {
			Ok(_) => Ok(()),
			Err(e) => Err(e.to_string()),
		}
	}

	fn initialize_environment(lua: &mut Lua, parameters: cli::Parameters)
	{
		let result = lua.context(|lua_ctx| -> rlua::Result<()> {
			let globals = lua_ctx.globals();
			let gr = lua_ctx.create_table().expect("Failed to create gr table");

			// Constructor for a SceneNode with no geometry
			let scene_node_ctor = lua_ctx
				.create_function(lua::SceneNode::lua_new)
				.expect("Failed to create node constructor");

			let nh_sphere_ctor = lua_ctx
				.create_function(Sphere::lua_new)
				.expect("Failed to create nh_sphere constructor");

			// NonhierBox Constructor
			let nh_box_ctor = lua_ctx
				.create_function(Cube::lua_new)
				.expect("Failed to create nh_box constructor");

			// Sphere Constructor
			let sphere_ctor = lua_ctx
				.create_function(Sphere::lua_unit_sphere)
				.expect("Failed to create sphere constructor");

			// Cube Constructor
			let cube_ctor = lua_ctx
				.create_function(Cube::lua_unit_cube)
				.expect("Failed to create cube constructor");

			// Mesh Constructor
			let mesh_ctor = lua_ctx
				.create_function(Mesh::lua_new)
				.expect("Failed to create mesh constructor");

			// Material Constructor
			let material_ctor = lua_ctx
				.create_function(Phong::lua_new)
				.expect("Failed to create mesh constructor");

			// CookTorrance Material Constructor
			let cook_torrance_ctor = lua_ctx
				.create_function(CookTorrance::lua_new)
				.expect("Failed to create mesh constructor");

			// Light Constructor
			let light_ctor = lua_ctx
				.create_function(Light::lua_new)
				.expect("Failed to create light constructor");

			// Render function
			let render = lua_ctx
				.create_function(
					move |lua_ctx,
					      (
						lua_scene_root,
						lua_output,
						lua_width,
						lua_height,
						lua_eye,
						lua_view,
						lua_up,
						lua_fov_y,
						lua_ambient,
						lua_lights,
					)| {
						SceneBuilder::lua_render(
							lua_ctx,
							lua_scene_root,
							lua_output,
							lua_width,
							lua_height,
							lua_eye,
							lua_view,
							lua_up,
							lua_fov_y,
							lua_ambient,
							lua_lights,
							&parameters,
						)
					},
				)
				.expect("Failed to create render function");

			gr.set("node", scene_node_ctor)
				.expect("Failed to assign LuaSceneNode constructor to gr.node");
			gr.set("nh_sphere", nh_sphere_ctor)
				.expect("Failed to assign NonhierSphere constructor to gr.nh_sphere");
			gr.set("nh_box", nh_box_ctor)
				.expect("Failed to assign NonhierBox constructor to gr.nh_box");
			gr.set("sphere", sphere_ctor)
				.expect("Failed to assign Sphere constructor to gr.sphere");
			gr.set("cube", cube_ctor)
				.expect("Failed to assign Cube constructor to gr.cube");
			gr.set("mesh", mesh_ctor)
				.expect("Failed to assign Mesh constructor to gr.mesh");
			gr.set("material", material_ctor)
				.expect("Failed to assign Material constructor to gr.material");
			gr.set("cook_torrance", cook_torrance_ctor)
				.expect("Failed to assign Material constructor to gr.material");
			gr.set("light", light_ctor)
				.expect("Failed to assign Light constructor to gr.light");
			gr.set("render", render)
				.expect("Failed to assign render function to gr.render");

			globals.set("gr", gr).expect("Failed to add gr to globals");

			Ok(())
		});

		result.unwrap();
	}

	fn lua_render<'lua>(
		lua: Context<'lua>,
		lua_scene_root: Value<'lua>,
		lua_output_name: Value<'lua>,
		lua_width: Value<'lua>,
		lua_height: Value<'lua>,
		lua_eye: Value<'lua>,
		lua_view: Value<'lua>,
		lua_up: Value<'lua>,
		lua_fov_y: Value<'lua>,
		lua_ambient: Value<'lua>,
		lua_lights: Value<'lua>,
		cli_parameters: &cli::Parameters,
	) -> rlua::Result<()>
	{
		let objects = match lua_scene_root {
			Value::UserData(user_data) => match user_data.borrow::<lua::SceneNode>() {
				Ok(root_node) => root_node.convert_to_object_list(),
				Err(error) => return Err(error),
			},
			_ => {
				return Err(rlua::Error::RuntimeError(
					"gr.render expected a scene node as its first argument".to_string(),
				))
			}
		};

		let output_name = String::from_lua(lua_output_name, lua)?;
		let width = u32::from_lua(lua_width, lua)?;
		let height = u32::from_lua(lua_height, lua)?;
		let eye = lua::Vector3::from_lua(lua_eye, lua)?;
		let view = lua::Vector3::from_lua(lua_view, lua)?;
		let up = lua::Vector3::from_lua(lua_up, lua)?;
		let fov_y = f32::from_lua(lua_fov_y, lua)?;
		let ambient = lua::Vector3::from_lua(lua_ambient, lua)?;

		let lights = match lua_lights {
			Value::Table(table) => {
				let mut vec = Vec::new();

				for value in table.sequence_values::<Light>() {
					let light = value?;
					vec.push(light);
				}

				vec
			}
			_ => {
				return Err(rlua::Error::RuntimeError(
					"gr.render expected an array as its tenth argument".to_string(),
				))
			}
		};

		let render_parameters = render::Parameters {
			objects: objects,
			lights: lights,
			output_file: cli_parameters.output_file.clone().unwrap_or(output_name),
			resolution: cli_parameters.resolution.unwrap_or((width, height)),
			eye_vector: na::Vector3::from(eye),
			view_vector: na::Vector3::from(view),
			up_vector: na::Vector3::from(up),
			vertical_fov: fov_y,
			ambient_light: na::Vector3::from(ambient),
			single_pixel: cli_parameters.single_pixel,
		};

		render::render(render_parameters);

		Ok(())
	}
}
