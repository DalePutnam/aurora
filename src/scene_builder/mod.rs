mod lua_scene_node;
mod lua_material;

use std::io::Read;
use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use na::Vector3;
use self::lua_scene_node::{LuaSceneNode, SceneNode};
use self::lua_material::LuaMaterial;
use core::traits::Primitive;
use core::{NonhierBox, NonhierSphere, Material};
use rlua::{Lua, UserData, UserDataMethods, Value};
use rlua;

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
        let size = match file.read_to_string(&mut contents) {
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
        node_constructor(lua_name)
    })
    .expect("Failed to create node constructor");

    // NonhierSphere Constructor
    let nh_sphere_ctor = lua.create_function(|_, (lua_name, lua_position, lua_radius): (Value, Value, Value)| {
        nh_sphere_constructor(lua_name, lua_position, lua_radius)
    })
    .expect("Failed to create nh_sphere constructor");

    // NonhierBox Constructor
    let nh_box_ctor = lua.create_function(|_, (lua_name, lua_position, lua_size): (Value, Value, Value)| {
        nh_box_constructor(lua_name, lua_position, lua_size)
    })
    .expect("Failed to create nh_box constructor");

    let material_ctor = lua.create_function(|_, (lua_diffuse, lua_specular, lua_shininess)| {
        material_constructor(lua_diffuse, lua_specular, lua_shininess)
    })
    .expect("Failed to create material constructor");

    gr.set("node", scene_node_ctor).expect("Failed to assign LuaSceneNode constructor to gr.node");
    gr.set("nh_sphere", nh_sphere_ctor).expect("Failed to assign NonhierSphere constructor to gr.nh_sphere");
    gr.set("nh_box", nh_box_ctor).expect("Failed to assign NonhierBox constructor to gr.nh_box");
    gr.set("material", material_ctor).expect("Failed to assign Material constructor to gr.material");

    globals.set("gr", gr).expect("Failed to add gr to globals");
}

fn node_constructor(lua_name: Value) -> rlua::Result<LuaSceneNode> {
    let name = match lua_name {
        Value::String(string) => string.to_str().unwrap().to_string(),
        _ => panic!("Failed to create node"),
    };

    Ok(LuaSceneNode::new(SceneNode::new(&name)))
}

fn nh_sphere_constructor(lua_name: Value, lua_position: Value, lua_radius: Value) -> rlua::Result<LuaSceneNode> {
    let name = match lua_name {
        Value::String(string) => string.to_str().unwrap().to_string(),
        _ => panic!("Failed to create nh_sphere"),
    };

    let position = match lua_position {
        Value::Table(table) => {
            if table.len()? != 3 {
                panic!("Invalid position given to nh_sphere constructor")
            }

            let x: f32 = table.get(1).unwrap();
            let y: f32 = table.get(2).unwrap();
            let z: f32 = table.get(3).unwrap();

            Vector3::<f32>::new(x, y, z)
        },
        _ => panic!("Failed to create nh_sphere")
    };

    let radius = match lua_radius {
        Value::Number(number) => number as f32,
        Value::Integer(integer) => integer as f32,
        _ => panic!("Failed to create nh_sphere"),
    };

    let mut node = SceneNode::new(&name);
    let nh_sphere: Arc<Box<Primitive>> = Arc::new(Box::new(NonhierSphere::new(position, radius)));

    node.set_primitive(&nh_sphere);

    Ok(LuaSceneNode::new(node))
}

fn nh_box_constructor(lua_name: Value, lua_position: Value, lua_size: Value) -> rlua::Result<LuaSceneNode> {
    let name = match lua_name {
        Value::String(string) => string.to_str().unwrap().to_string(),
        _ => panic!("Failed to create nh_box"),
    };

    let position = match lua_position {
        Value::Table(table) => {
            if table.len().unwrap() != 3 {
                panic!("Invalid position given to nh_box constructor")
            }

            let x: f32 = table.get(1).unwrap();
            let y: f32 = table.get(2).unwrap();
            let z: f32 = table.get(3).unwrap();

            Vector3::<f32>::new(x, y, z)
        },
        _ => panic!("Failed to create nh_box")
    };

    let size = match lua_size {
        Value::Number(number) => number as f32,
        Value::Integer(integer) => integer as f32,
        _ => panic!("Failed to create nh_box"),
    };

    let mut node = SceneNode::new(&name);
    let nh_box: Arc<Box<Primitive>> = Arc::new(Box::new(NonhierBox::new(position, size)));

    node.set_primitive(&nh_box);

    Ok(LuaSceneNode::new(node))    
}

fn material_constructor(lua_diffuse: Value, lua_specular: Value, lua_shininess: Value) -> rlua::Result<LuaMaterial> {
    let diffuse = match lua_diffuse {
        Value::Table(table) => {
            if table.len()? != 3 {
                return Err(rlua::Error::RuntimeError("gr.material expected an array with 3 elements as its first argument".to_string()));
            }

            let x: f32 = table.get(1)?;
            let y: f32 = table.get(2)?;
            let z: f32 = table.get(3)?;

            Vector3::<f32>::new(x, y, z)
        },
        _ => return Err(rlua::Error::RuntimeError("gr.material expected an array as its first argument".to_string())),
    };

    let specular = match lua_specular {
        Value::Table(table) => {
            if table.len()? != 3 {
                return Err(rlua::Error::RuntimeError("gr.material expected an array with 3 elements as its second argument".to_string()));
            }

            let x: f32 = table.get(1)?;
            let y: f32 = table.get(2)?;
            let z: f32 = table.get(3)?;

            Vector3::<f32>::new(x, y, z)
        },
        _ => return Err(rlua::Error::RuntimeError("gr.material expected an array as its second argument".to_string())),
    };

    let shininess = match lua_shininess {
        Value::Number(number) => number as f32,
        Value::Integer(integer) => integer as f32,
        _ => return Err(rlua::Error::RuntimeError("gr.material expected an Integer or a Number as its third argument".to_string())),
    };

    Ok(LuaMaterial::new(Material::new(diffuse, specular, shininess)))
}