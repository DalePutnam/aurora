use std::sync::Arc;
use core::Light;
use rlua;
use rlua::{UserData, UserDataMethods, Value};
use na::Vector3;

pub struct LuaLight {
    light: Arc<Light>,
}

impl LuaLight {
    pub fn new(light: Light) -> Self {
        LuaLight {
            light: Arc::new(light),
        }
    }
}

impl UserData for LuaLight {
    fn add_methods(_methods: &mut UserDataMethods<Self>) {

    }
}

pub fn lua_light_constructor(lua_position: Value, lua_colour: Value, lua_falloff: Value) -> rlua::Result<LuaLight> {
    let position = match lua_position {
        Value::Table(table) => {
            if table.len()? != 3 {
                return Err(rlua::Error::RuntimeError("gr.light expected an array with 3 elements as its first argument".to_string()));
            }

            let x: f32 = table.get(1)?;
            let y: f32 = table.get(2)?;
            let z: f32 = table.get(3)?;

            Vector3::<f32>::new(x, y, z)
        },
        _ => return Err(rlua::Error::RuntimeError("gr.light expected an array as its first argument".to_string())),
    };

    let colour = match lua_colour {
        Value::Table(table) => {
            if table.len()? != 3 {
                return Err(rlua::Error::RuntimeError("gr.light expected an array with 3 elements as its second argument".to_string()));
            }

            let x: f32 = table.get(1)?;
            let y: f32 = table.get(2)?;
            let z: f32 = table.get(3)?;

            Vector3::<f32>::new(x, y, z)
        },
        _ => return Err(rlua::Error::RuntimeError("gr.light expected an array as its second argument".to_string())),
    };

    let falloff = match lua_falloff {
        Value::Table(table) => {
            if table.len()? != 3 {
                return Err(rlua::Error::RuntimeError("gr.light expected an array with 3 elements as its third argument".to_string()));
            }

            let x: f32 = table.get(1)?;
            let y: f32 = table.get(2)?;
            let z: f32 = table.get(3)?;

            Vector3::<f32>::new(x, y, z)
        },
        _ => return Err(rlua::Error::RuntimeError("gr.light expected an array as its third argument".to_string())),
    };

    Ok(LuaLight::new(Light::new(&position, &colour, &falloff)))
}