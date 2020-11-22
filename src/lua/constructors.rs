use lua;
use na;
use rlua::{Context, FromLua, Value};
use Cube;
use Light;
use Material;
use Mesh;
use NonhierBox;
use NonhierSphere;
use Sphere;
use Phong;

impl Phong {
    pub fn lua_new<'lua>(
        lua: Context<'lua>,
        lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
    ) -> rlua::Result<lua::Pointer<Material>> {
        let (lua_diffuse, lua_specular, lua_shininess) = lua_value;

        let diffuse = lua::Vector3::from_lua(lua_diffuse, lua)?;
        let specular = lua::Vector3::from_lua(lua_specular, lua)?;
        let shininess = f32::from_lua(lua_shininess, lua)?;

        Ok(lua::Pointer::new(Material::new(Phong::new(
            na::Vector3::from(diffuse),
            na::Vector3::from(specular),
            shininess,
        ))))
    }
}

impl NonhierSphere {
    pub fn lua_new<'lua>(
        lua: Context<'lua>,
        lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
    ) -> rlua::Result<lua::SceneNode> {
        let (lua_name, lua_position, lua_radius) = lua_value;

        let name = String::from_lua(lua_name, lua)?;
        let position = lua::Vector3::from_lua(lua_position, lua)?;
        let radius = f32::from_lua(lua_radius, lua)?;

        let mut node = lua::SceneNode::new(&name);
        let nh_sphere = lua::Pointer::new(NonhierSphere::new(position.into(), radius));

        node.set_primitive(nh_sphere);

        Ok(node)
    }
}

impl NonhierBox {
    pub fn lua_new<'lua>(
        lua: Context<'lua>,
        lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
    ) -> rlua::Result<lua::SceneNode> {
        let (lua_name, lua_position, lua_size) = lua_value;

        let name = String::from_lua(lua_name, lua)?;
        let position = lua::Vector3::from_lua(lua_position, lua)?;
        let size = f32::from_lua(lua_size, lua)?;

        let mut node = lua::SceneNode::new(&name);
        let nh_box = lua::Pointer::new(NonhierBox::new(position.into(), size));

        node.set_primitive(nh_box);

        Ok(node)
    }
}

impl Sphere {
    pub fn lua_new<'lua>(
        lua: Context<'lua>,
        lua_name: Value<'lua>,
    ) -> rlua::Result<lua::SceneNode> {
        let name = String::from_lua(lua_name, lua)?;

        let mut node = lua::SceneNode::new(&name);
        let sphere = lua::Pointer::new(Sphere::new());

        node.set_primitive(sphere);

        Ok(node)
    }
}

impl Cube {
    pub fn lua_new<'lua>(
        lua: Context<'lua>,
        lua_name: Value<'lua>,
    ) -> rlua::Result<lua::SceneNode> {
        let name = String::from_lua(lua_name, lua)?;

        let mut node = lua::SceneNode::new(&name);
        let sphere = lua::Pointer::new(Cube::new());

        node.set_primitive(sphere);

        Ok(node)
    }
}

impl Mesh {
    pub fn lua_new<'lua>(
        lua: Context<'lua>,
        lua_value: (Value<'lua>, Value<'lua>),
    ) -> rlua::Result<lua::SceneNode> {
        let (lua_name, lua_file_name) = lua_value;

        let name = String::from_lua(lua_name, lua)?;
        let file_name = String::from_lua(lua_file_name, lua)?;

        let mut node = lua::SceneNode::new(&name);
        let mesh = lua::Pointer::new(Mesh::new(&file_name));

        node.set_primitive(mesh);

        Ok(node)
    }
}

impl Light {
    pub fn lua_new<'lua>(
        lua: Context<'lua>,
        lua_value: (Value<'lua>, Value<'lua>, Value<'lua>),
    ) -> rlua::Result<Light> {
        let (lua_position, lua_colour, lua_falloff) = lua_value;

        let position = lua::Vector3::from_lua(lua_position, lua)?;
        let colour = lua::Vector3::from_lua(lua_colour, lua)?;
        let falloff = lua::Vector3::from_lua(lua_falloff, lua)?;

        Ok(Light::new(
            &na::Vector3::from(position),
            &na::Vector3::from(colour),
            &na::Vector3::from(falloff),
        ))
    }
}
