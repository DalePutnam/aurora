use na::Vector3;
use std::sync::Arc;
use lua::vector3;
use rlua::{Context, Value, UserData, UserDataMethods, FromLua};

struct MaterialInner {
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
    shininess: f32,
}

#[derive(Clone)]
pub struct Material
{
    inner: Arc<MaterialInner>
}

impl Material {
    pub fn new(diffuse: Vector3<f32>, specular: Vector3<f32>, shininess: f32) -> Self {
        Material {
            inner: Arc::new(MaterialInner {
                        diffuse: diffuse,
                        specular: specular,
                        shininess: shininess,
                    })
        }
    }

    pub fn get_diffuse(&self) -> &Vector3<f32> {
        &self.inner.diffuse
    }

    pub fn get_specular(&self) -> &Vector3<f32> {
        &self.inner.specular
    }

    pub fn get_shininess(&self) -> f32 {
        self.inner.shininess
    }

    pub fn lua_new<'lua>(lua: Context<'lua>, lua_value: (Value<'lua>, Value<'lua>, Value<'lua>)) -> rlua::Result<Material> {
        let (lua_diffuse, lua_specular, lua_shininess) = lua_value;

        let diffuse = vector3::from_lua(lua_diffuse, lua)?;
        let specular = vector3::from_lua(lua_specular, lua)?;
        let shininess = f32::from_lua(lua_shininess, lua)?;
    
        Ok(Material::new(diffuse, specular, shininess))
    }
}

impl UserData for Material {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(_methods: &mut T) {}
}