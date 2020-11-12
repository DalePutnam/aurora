use std::sync::Arc;
use core::Material;
use rlua::{self, UserData, UserDataMethods, Value, FromLua, Context};
use scene_builder::LuaVector3;

pub struct LuaMaterial {
    material: Arc<Box<Material>>,
}

impl LuaMaterial {
    pub fn get_inner(&self) -> &Arc<Box<Material>> {
        &self.material
    }

    fn new(material: Box<Material>) -> Self {
        LuaMaterial {
            material: Arc::new(material),
        }
    }
}

impl UserData for LuaMaterial {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(_methods: &mut T) {}
}

pub fn lua_material_constructor<'lua>(lua: Context<'lua>, lua_diffuse: Value<'lua>, lua_specular: Value<'lua>, lua_shininess: Value<'lua>) -> rlua::Result<LuaMaterial> {
    let diffuse = LuaVector3::from_lua(lua_diffuse, lua)?.get_inner();
    let specular = LuaVector3::from_lua(lua_specular, lua)?.get_inner();
    let shininess = f32::from_lua(lua_shininess, lua)?;

    Ok(LuaMaterial::new(Box::new(Material::new(diffuse, specular, shininess))))
}