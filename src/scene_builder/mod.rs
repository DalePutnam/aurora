mod lua_scene_node;
mod lua_material;
mod lua_light;

use std::io::Read;
use std::error::Error;
use std::fs::File;
use rlua::{Lua, Value};

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

        match self.lua.exec::<()>(&contents, Some(script_path)) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Lua execution failed: {}", e)),
        }
    }
}

fn initialize_environment(lua: &mut Lua) {
    let globals = lua.globals();
    let gr = lua.create_table().expect("Failed to create gr table");

    // Constructor for a SceneNode with no geometry
    let scene_node_ctor = lua.create_function(|_, lua_name: Value| {
        lua_scene_node::lua_node_constructor(lua_name)
    })
    .expect("Failed to create node constructor");

    // NonhierSphere Constructor
    let nh_sphere_ctor = lua.create_function(|_, (lua_name, lua_position, lua_radius): (Value, Value, Value)| {
        lua_scene_node::lua_nh_sphere_constructor(lua_name, lua_position, lua_radius)
    })
    .expect("Failed to create nh_sphere constructor");

    // NonhierBox Constructor
    let nh_box_ctor = lua.create_function(|_, (lua_name, lua_position, lua_size): (Value, Value, Value)| {
        lua_scene_node::lua_nh_box_constructor(lua_name, lua_position, lua_size)
    })
    .expect("Failed to create nh_box constructor");

    // Material Constructor
    let material_ctor = lua.create_function(|_, (lua_diffuse, lua_specular, lua_shininess)| {
        lua_material::lua_material_constructor(lua_diffuse, lua_specular, lua_shininess)
    })
    .expect("Failed to create material constructor");

    // Light Constructor
    let light_ctor = lua.create_function(|_, (lua_position, lua_colour, lua_falloff)| {
        lua_light::lua_light_constructor(lua_position, lua_colour, lua_falloff)
    })
    .expect("Failed to create light constructor");

    gr.set("node", scene_node_ctor).expect("Failed to assign LuaSceneNode constructor to gr.node");
    gr.set("nh_sphere", nh_sphere_ctor).expect("Failed to assign NonhierSphere constructor to gr.nh_sphere");
    gr.set("nh_box", nh_box_ctor).expect("Failed to assign NonhierBox constructor to gr.nh_box");
    gr.set("material", material_ctor).expect("Failed to assign Material constructor to gr.material");
    gr.set("light", light_ctor).expect("Failed to assign Light constructor to gr.light");

    globals.set("gr", gr).expect("Failed to add gr to globals");
}
