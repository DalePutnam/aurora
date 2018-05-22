pub use self::lua_scene_node::LuaSceneNode;
pub use self::lua_material::LuaMaterial;
pub use self::lua_light::LuaLight;

mod lua_scene_node;
mod lua_material;
mod lua_light;

use std::io::Read;
use std::error::Error;
use std::fs::File;
use rlua::{Lua, Value};
use rlua;
use na::Vector3;
use core;

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

    // Render function
    let render = lua.create_function(|_, (lua_scene_root, lua_output, lua_width, lua_height, lua_eye, lua_view, lua_up, lua_fov_y, lua_ambient, lua_lights)| {
        lua_render(lua_scene_root, lua_output, lua_width, lua_height, lua_eye, lua_view, lua_up, lua_fov_y, lua_ambient, lua_lights)
    })
    .expect("Failed to create render function");

    gr.set("node", scene_node_ctor).expect("Failed to assign LuaSceneNode constructor to gr.node");
    gr.set("nh_sphere", nh_sphere_ctor).expect("Failed to assign NonhierSphere constructor to gr.nh_sphere");
    gr.set("nh_box", nh_box_ctor).expect("Failed to assign NonhierBox constructor to gr.nh_box");
    gr.set("material", material_ctor).expect("Failed to assign Material constructor to gr.material");
    gr.set("light", light_ctor).expect("Failed to assign Light constructor to gr.light");
    gr.set("render", render).expect("Failed to assign render function to gr.render");

    globals.set("gr", gr).expect("Failed to add gr to globals");
}

fn lua_render(lua_scene_root: Value, lua_output_name: Value, lua_width: Value, lua_height: Value,
            lua_eye: Value, lua_view: Value, lua_up: Value, lua_fov_y: Value, lua_ambient: Value, lua_lights: Value) -> rlua::Result<()> {

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

    let output_name = match lua_output_name {
        Value::String(string) => string.to_str().unwrap().to_string(),
        _ => return Err(rlua::Error::RuntimeError("gr.render expected a string as its second argument".to_string())),
    };

    let width = match lua_width {
        Value::Integer(i) => i as u32,
        Value::Number(n) => n as u32,
        _ => return Err(rlua::Error::RuntimeError("gr.render expected a number as its third argument".to_string())),
    };

    let height = match lua_height {
        Value::Integer(i) => i as u32,
        Value::Number(n) => n as u32,
        _ => return Err(rlua::Error::RuntimeError("gr.render expected a number as its fourth argument".to_string())),
    };

    let eye = match lua_eye {
        Value::Table(table) => {
            if table.len()? != 3 {
                return Err(rlua::Error::RuntimeError("gr.render expected an array with 3 elements as its fifth argument".to_string()));
            }

            let x: f32 = table.get(1)?;
            let y: f32 = table.get(2)?;
            let z: f32 = table.get(3)?;

            Vector3::<f32>::new(x, y, z)
        },
        _ => return Err(rlua::Error::RuntimeError("gr.render expected an array with 3 elements as its fifth argument".to_string())),
    };

    let view = match lua_view {
        Value::Table(table) => {
            if table.len()? != 3 {
                return Err(rlua::Error::RuntimeError("gr.render expected an array with 3 elements as its sixth argument".to_string()));
            }

            let x: f32 = table.get(1)?;
            let y: f32 = table.get(2)?;
            let z: f32 = table.get(3)?;

            Vector3::<f32>::new(x, y, z)
        },
        _ => return Err(rlua::Error::RuntimeError("gr.render expected an array with 3 elements as its sixth argument".to_string())),
    };

    let up = match lua_up {
        Value::Table(table) => {
            if table.len()? != 3 {
                return Err(rlua::Error::RuntimeError("gr.render expected an array with 3 elements as its seventh argument".to_string()));
            }

            let x: f32 = table.get(1)?;
            let y: f32 = table.get(2)?;
            let z: f32 = table.get(3)?;

            Vector3::<f32>::new(x, y, z)
        },
        _ => return Err(rlua::Error::RuntimeError("gr.render expected an array with 3 elements as its seventh argument".to_string())),
    };

    let fov_y = match lua_fov_y {
        Value::Integer(i) => i as f32,
        Value::Number(n) => n as f32,
        _ => return Err(rlua::Error::RuntimeError("gr.render expected a number as its eighth argument".to_string())),
    };

    let ambient = match lua_ambient {
        Value::Table(table) => {
            if table.len()? != 3 {
                return Err(rlua::Error::RuntimeError("gr.render expected an array with 3 elements as its ninth argument".to_string()));
            }

            let x: f32 = table.get(1)?;
            let y: f32 = table.get(2)?;
            let z: f32 = table.get(3)?;

            Vector3::<f32>::new(x, y, z)
        },
        _ => return Err(rlua::Error::RuntimeError("gr.render expected an array with 3 elements as its ninth argument".to_string())),
    };

    let lights = match lua_lights {
        Value::Table(table) => {
            let mut vec = Vec::new();

            let mut i = 0;
            while table.contains_key(i)? {
                vec.push(table.get::<u32, LuaLight>(i)?.get_internal_light());
                i += 1;
            }

            vec
        },
        _ => return Err(rlua::Error::RuntimeError("gr.render expected an array as its tenth argument".to_string())),
    };

    core::render(objects, output_name, width, height, eye, view, up, fov_y, ambient, lights);

    Ok(())
}
