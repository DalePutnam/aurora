pub use self::lua_scene_node::LuaSceneNode;
pub use self::lua_material::LuaMaterial;
pub use self::lua_light::LuaLight;
pub use self::lua_vector::LuaVector3;

mod lua_vector;
mod lua_scene_node;
mod lua_material;
mod lua_light;

use std::io::Read;
use std::error::Error;
use std::fs::File;
use rlua::{self, Lua, Value, FromLua, Context};
use core;
use failure::Fail;

pub struct SceneBuilder {
    lua: Lua,
}

impl SceneBuilder {
    pub fn new() -> Self {
        let mut lua = Lua::new();
        initialize_environment(&mut lua);

        SceneBuilder {
            lua: lua,
        }
    }

    pub fn run_build_script(&self, script_path: &String) -> Result<(), String> {
        let mut file = match File::open(script_path) {
            Ok(f) => f,
            Err(e) => return Err(e.description().to_string()),
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(s) => s,
            Err(e) => return Err(e.description().to_string()),
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
}

fn initialize_environment(lua: &mut Lua) {
    let result = lua.context(|lua_ctx| -> rlua::Result<()> {
        let globals = lua_ctx.globals();
        let gr = lua_ctx.create_table().expect("Failed to create gr table");

        // Constructor for a SceneNode with no geometry
        let scene_node_ctor = lua_ctx.create_function(|lua_ctx, lua_name: Value| {
            lua_scene_node::lua_node_constructor(lua_ctx, lua_name)
        })
        .expect("Failed to create node constructor");

        // NonhierSphere Constructor
        let nh_sphere_ctor = lua_ctx.create_function(|lua_ctx, (lua_name, lua_position, lua_radius)| {
            lua_scene_node::lua_nh_sphere_constructor(lua_ctx, lua_name, lua_position, lua_radius)
        })
        .expect("Failed to create nh_sphere constructor");

        // NonhierBox Constructor
        let nh_box_ctor = lua_ctx.create_function(|lua_ctx, (lua_name, lua_position, lua_size)| {
            lua_scene_node::lua_nh_box_constructor(lua_ctx, lua_name, lua_position, lua_size)
        })
        .expect("Failed to create nh_box constructor");

        // Sphere Constructor
        let sphere_ctor = lua_ctx.create_function(|lua_ctx, lua_name| {
            lua_scene_node::lua_sphere_constructor(lua_ctx, lua_name)
        })
        .expect("Failed to create sphere constructor");

        // Cube Constructor
        let cube_ctor = lua_ctx.create_function(|lua_ctx, lua_name| {
            lua_scene_node::lua_cube_constructor(lua_ctx, lua_name)
        })
        .expect("Failed to create cube constructor");

        // Mesh Constructor
        let mesh_ctor = lua_ctx.create_function(|lua_ctx, (lua_name, lua_file_name)| {
            lua_scene_node::lua_mesh_constructor(lua_ctx, lua_name, lua_file_name)
        })
        .expect("Failed to create mesh constructor");

        // Material Constructor
        let material_ctor = lua_ctx.create_function(|lua_ctx, (lua_diffuse, lua_specular, lua_shininess)| {
            lua_material::lua_material_constructor(lua_ctx, lua_diffuse, lua_specular, lua_shininess)
        })
        .expect("Failed to create material constructor");

        // Light Constructor
        let light_ctor = lua_ctx.create_function(|lua_ctx, (lua_position, lua_colour, lua_falloff)| {
            lua_light::lua_light_constructor(lua_ctx, lua_position, lua_colour, lua_falloff)
        })
        .expect("Failed to create light constructor");

        // Render function
        let render = lua_ctx.create_function(|lua_ctx, (lua_scene_root, lua_output, lua_width, lua_height, lua_eye, lua_view, lua_up, lua_fov_y, lua_ambient, lua_lights)| {
            lua_render(lua_ctx, lua_scene_root, lua_output, lua_width, lua_height, lua_eye, lua_view, lua_up, lua_fov_y, lua_ambient, lua_lights)
        })
        .expect("Failed to create render function");

        gr.set("node", scene_node_ctor).expect("Failed to assign LuaSceneNode constructor to gr.node");
        gr.set("nh_sphere", nh_sphere_ctor).expect("Failed to assign NonhierSphere constructor to gr.nh_sphere");
        gr.set("nh_box", nh_box_ctor).expect("Failed to assign NonhierBox constructor to gr.nh_box");
        gr.set("sphere", sphere_ctor).expect("Failed to assign Sphere constructor to gr.sphere");
        gr.set("cube", cube_ctor).expect("Failed to assign Cube constructor to gr.cube");
        gr.set("mesh", mesh_ctor).expect("Failed to assign Mesh constructor to gr.mesh");
        gr.set("material", material_ctor).expect("Failed to assign Material constructor to gr.material");
        gr.set("light", light_ctor).expect("Failed to assign Light constructor to gr.light");
        gr.set("render", render).expect("Failed to assign render function to gr.render");

        globals.set("gr", gr).expect("Failed to add gr to globals");
   
        Ok(())
    });

    result.unwrap();
}

fn lua_render<'lua>(lua: Context<'lua>, lua_scene_root: Value<'lua>, lua_output_name: Value<'lua>, lua_width: Value<'lua>, lua_height: Value<'lua>,
            lua_eye: Value<'lua>, lua_view: Value<'lua>, lua_up: Value<'lua>, lua_fov_y: Value<'lua>, lua_ambient: Value<'lua>, lua_lights: Value<'lua>) -> rlua::Result<()> {

    let objects = match lua_scene_root {
        Value::UserData(user_data) => {
            match user_data.borrow::<LuaSceneNode>() {
                Ok(n) => {
                    n.convert_to_object_list()
                },
                Err(e) => return Err(e), 
            }
        },
        _ => return Err(rlua::Error::RuntimeError("gr.render expected a scene node as its first argument".to_string())),
    };

    let output_name = String::from_lua(lua_output_name, lua)?;
    let width = u32::from_lua(lua_width, lua)?;
    let height = u32::from_lua(lua_height, lua)?;
    let eye = LuaVector3::from_lua(lua_eye, lua)?.get_inner();
    let view = LuaVector3::from_lua(lua_view, lua)?.get_inner();
    let up = LuaVector3::from_lua(lua_up, lua)?.get_inner();
    let fov_y = f32::from_lua(lua_fov_y, lua)?;
    let ambient = LuaVector3::from_lua(lua_ambient, lua)?.get_inner();

    let lights = match lua_lights {
        Value::Table(table) => {
            let mut vec = Vec::new();

            for value in table.sequence_values::<LuaLight>() {
                let light = value?;
                vec.push(light.get_internal_light());
            }

            vec
        },
        _ => return Err(rlua::Error::RuntimeError("gr.render expected an array as its tenth argument".to_string())),
    };

    core::render(objects, output_name, width, height, eye, view, up, fov_y, ambient, lights);

    Ok(())
}
