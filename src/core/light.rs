use na::{Vector4, Vector3};
use core::lua::vector3;
use std::sync::Arc;
use rlua::{Context, Value, UserData, UserDataMethods};

struct LightInner {
    position: Vector4<f32>,
    colour: Vector3<f32>,
    falloff: Vector3<f32>,
}

#[derive(Clone)]
pub struct Light {
    inner: Arc<LightInner>
}

impl Light {
    pub fn new(position: &Vector3<f32>, colour: &Vector3<f32>, falloff: &Vector3<f32>) -> Self {
        Light {
            inner: 
                Arc::new(LightInner {
                    position: Vector4::<f32>::new(position.x, position.y, position.z, 1.0),
                    colour: *colour,
                    falloff: *falloff,
                })
        }
    }

    pub fn get_position(&self) -> &Vector4<f32> {
        &self.inner.position
    }

    pub fn get_colour(&self) -> &Vector3<f32> {
        &self.inner.colour
    }

    pub fn get_falloff(&self) -> &Vector3<f32> {
        &self.inner.falloff
    }

    pub fn lua_new<'lua>(lua: Context<'lua>, lua_value: (Value<'lua>, Value<'lua>, Value<'lua>)) -> rlua::Result<Light> {
        let (lua_position, lua_colour, lua_falloff) = lua_value;

        let position = vector3::from_lua(lua_position, lua)?;
        let colour =  vector3::from_lua(lua_colour, lua)?;
        let falloff = vector3::from_lua(lua_falloff, lua)?;
    
        Ok(Light::new(&position, &colour, &falloff))
    }
}

impl UserData for Light {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(_methods: &mut T) {}
}