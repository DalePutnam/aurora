use std::sync::Arc;
use core::Material;
use rlua;
use rlua::{UserData, UserDataMethods, Value};
use na::Vector3;

pub struct LuaMaterial {
    material: Arc<Material>,
}

impl LuaMaterial {
    pub fn new(material: Material) -> Self {
        LuaMaterial {
            material: Arc::new(material),
        }
    }

    pub fn get_internal_material(&self) -> &Arc<Material> {
        &self.material
    }
}

impl UserData for LuaMaterial {
    fn add_methods(_methods: &mut UserDataMethods<Self>) {

    }
}

pub fn lua_material_constructor(lua_diffuse: Value, lua_specular: Value, lua_shininess: Value) -> rlua::Result<LuaMaterial> {
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