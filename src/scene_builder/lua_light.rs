use std::sync::Arc;
use core::Light;
use rlua::{self, UserData, UserDataMethods, Value, FromLua, Context};
use scene_builder::LuaVector3;

#[derive(Clone)]
pub struct LuaLight {
    light: Arc<Light>,
}

impl LuaLight {
    pub fn new(light: Light) -> Self {
        LuaLight {
            light: Arc::new(light),
        }
    }

    pub fn get_internal_light(&self) -> Light {
        Light::clone(&self.light)
    }
}

impl UserData for LuaLight {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(_methods: &mut T) {}
}

pub fn lua_light_constructor<'lua>(lua: Context<'lua>, lua_position: Value<'lua>, lua_colour: Value<'lua>, lua_falloff: Value<'lua>) -> rlua::Result<LuaLight> {
    let position = LuaVector3::from_lua(lua_position, lua)?;
    let colour = LuaVector3::from_lua(lua_colour, lua)?;
    let falloff = LuaVector3::from_lua(lua_falloff, lua)?;

    Ok(LuaLight::new(Light::new(&position.get_inner(), &colour.get_inner(), &falloff.get_inner())))
}