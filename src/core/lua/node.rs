use na::{Matrix4, Vector3, Unit};
use rlua::{self, UserData, UserDataMethods, Value, FromLua, Context};
use core::{Object, Material, PrimitivePtr};
use core::lua::vector3;
use core::{Mesh, Cube, Sphere, NonhierBox, NonhierSphere};
use std::sync::{Arc, RwLock};
use std::cell::RefCell;
use std::ops::Deref;

pub struct SceneNode {
    _name: String,
    transform: Matrix4<f32>,
    children: Vec<SceneNodePtr>,
    primitive: Option<PrimitivePtr>,
    material: Option<Material>
}

impl SceneNode {
    pub fn new(name: &str) -> Self {
        SceneNode { 
            _name: name.to_string(),
            transform: Matrix4::identity(),
            children: Vec::new(),
            primitive: None,
            material: None,
        }
    }

    pub fn convert_to_object_list(&self, list: &RefCell<Vec<Object>>, transform: &Matrix4<f32>, current_id: &mut u64) {
        let new_transform = transform * self.transform;

        if let Some(ref primitive) = self.primitive {
            if let Some(ref material) = self.material {
                list.borrow_mut().push(Object::new(*current_id, &new_transform, &primitive, &material));
            }
        }

        let mut id = *current_id + 1;

        for node_lock in &self.children {
            match node_lock.read() {
                Ok(node) => {
                    node.convert_to_object_list(list, &new_transform, &mut id);
                },
                Err(_) => panic!("Child lock is poisoned!"),
            };
        }

        *current_id = id;
    }

    pub fn set_primitive(&mut self, primitive: &PrimitivePtr) {
        self.primitive = Some(primitive.clone());
    }

    pub fn set_material(&mut self, material: &Material) {
        self.material = Some(material.clone());
    }

    pub fn rotate(&mut self, axis: char, angle: f32) {
        let rotation_axis = match axis {
            'x' | 'X' => Vector3::new(1.0, 0.0, 0.0),
            'y' | 'Y' => Vector3::new(0.0, 1.0, 0.0),
            'z' | 'Z' => Vector3::new(0.0, 0.0, 1.0),
             _  => return,
        };

        let rotation_matrix = Matrix4::from_axis_angle(&Unit::new_normalize(rotation_axis), angle.to_radians());
        self.transform = rotation_matrix * self.transform;
    }

    pub fn scale(&mut self, amount: &Vector3<f32>) {
        let scale_matrix = Matrix4::new_nonuniform_scaling(amount);
        self.transform = scale_matrix * self.transform;
    }

    pub fn translate(&mut self, amount: &Vector3<f32>) {
        let translate_matrix = Matrix4::new_translation(amount);
        self.transform = translate_matrix * self.transform;
    }

    pub fn add_child(&mut self, node: &SceneNodePtr) {
        self.children.push(node.clone());
    }
}

#[derive(Clone)]
pub struct SceneNodePtr {
    inner: Arc<RwLock<SceneNode>>
}

impl SceneNodePtr {
    pub fn new(scene_node: SceneNode) -> Self {
        SceneNodePtr { inner: Arc::new(RwLock::new(scene_node)) }
    }

    /// Creates a vector of Objects from the scene tree represented by this LuaSceneNode
    pub fn convert_to_object_list(&self) -> Vec<Object> {
        let list = RefCell::new(Vec::new());
        let mut id = 0;

        match self.read() {
            Ok(scene_node) => {
                scene_node.convert_to_object_list(&list, &Matrix4::identity(), &mut id);
            },
            Err(_) => panic!("Node lock is poisoned!"),
        };

        list.into_inner()
    }

}

impl Deref for SceneNodePtr {
    type Target = Arc<RwLock<SceneNode>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl UserData for SceneNodePtr {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method_mut("rotate", |lua, lua_node, (lua_axis, lua_angle)| {
            let axis = match lua_axis {
                Value::String(string) => {
                    match string.to_str() {
                        Ok(slice) => slice.chars().nth(0).unwrap(),
                        Err(e) => panic!(e),
                    }
                },
                _ => panic!("Failed to rotate"),
            };

            let angle = f32::from_lua(lua_angle, lua)?;

            match lua_node.write() {
                Ok(mut scene_node) => scene_node.rotate(axis, angle),
                Err(_) => panic!("SceneNode lock is poisoned!"),
            };

            Ok(())
        });

        methods.add_method("scale", |lua, lua_node, (lua_x, lua_y, lua_z)| {
            let x = f32::from_lua(lua_x, lua)?;
            let y = f32::from_lua(lua_y, lua)?;
            let z = f32::from_lua(lua_z, lua)?;
            let amount = Vector3::new(x, y, z);

            match lua_node.write() {
                Ok(mut scene_node) => scene_node.scale(&amount),
                Err(_) => panic!("SceneNode lock is poisoned!"),
            };

            Ok(())
        });

        methods.add_method("translate", |lua, lua_node, (lua_x, lua_y, lua_z)| {
            let x = f32::from_lua(lua_x, lua)?;
            let y = f32::from_lua(lua_y, lua)?;
            let z = f32::from_lua(lua_z, lua)?;
            let amount = Vector3::new(x, y, z);

            match lua_node.write() {
                Ok(mut scene_node) => scene_node.translate(&amount),
                Err(_) => panic!("SceneNode lock is poisoned!"),
            };

            Ok(())
        });

        methods.add_method("add_child", |_, lua_node, child_lua_node| {
            match child_lua_node {
                Value::UserData(user_data) => {
                    match user_data.borrow::<SceneNodePtr>() {
                        Ok(child_node) => {
                            match lua_node.write() {
                                Ok(mut scene_node) => scene_node.add_child(&child_node),
                                Err(_) => panic!("SceneNode lock is poisoned!"),
                            };

                            Ok(())
                        },
                        Err(_) => panic!("Invalid node!"),
                    }
                },
                _ => panic!("Invalid node!"),
            }
        });

        methods.add_method("set_material", |_, lua_node, lua_material| {
            match lua_material {
                Value::UserData(user_data) => {
                    match user_data.borrow::<Material>() {
                        Ok(material) => {
                            match lua_node.write() {
                                Ok(mut scene_node) => scene_node.set_material(&material),
                                Err(_) => panic!("SceneNode lock is poisoned"),
                            };

                            Ok(())
                        },
                        Err(e) => Err(e),
                    }
                },
                _ => Err(rlua::Error::RuntimeError("set_material expected a LuaMaterial as its first argument".to_string())),
            }
        });
    }
}

pub fn lua_node_constructor<'lua>(lua: Context<'lua>, lua_name: Value<'lua>) -> rlua::Result<SceneNodePtr> {
    let name = String::from_lua(lua_name, lua)?;
    Ok(SceneNodePtr::new(SceneNode::new(&name)))
}

pub fn lua_nh_sphere_constructor<'lua>(lua:  Context<'lua>, lua_name: Value<'lua>, lua_position: Value<'lua>, lua_radius: Value<'lua>) -> rlua::Result<SceneNodePtr> {
    let name = String::from_lua(lua_name, lua)?;
    let position = vector3::from_lua(lua_position, lua)?;
    let radius = f32::from_lua(lua_radius, lua)?;

    let mut node = SceneNode::new(&name);
    let nh_sphere= PrimitivePtr::new(NonhierSphere::new(position, radius));

    node.set_primitive(&nh_sphere);

    Ok(SceneNodePtr::new(node))
}

pub fn lua_nh_box_constructor<'lua>(lua: Context<'lua>, lua_name: Value<'lua>, lua_position: Value<'lua>, lua_size: Value<'lua>) -> rlua::Result<SceneNodePtr> {
    let name = String::from_lua(lua_name, lua)?;
    let position = vector3::from_lua(lua_position, lua)?;
    let size = f32::from_lua(lua_size, lua)?;

    let mut node = SceneNode::new(&name);
    let nh_box= PrimitivePtr::new(NonhierBox::new(position, size));

    node.set_primitive(&nh_box);

    Ok(SceneNodePtr::new(node))    
}

pub fn lua_sphere_constructor<'lua>(lua: Context<'lua>, lua_name: Value<'lua>) -> rlua::Result<SceneNodePtr> {
    let name = String::from_lua(lua_name, lua)?;

    let mut node = SceneNode::new(&name);
    let sphere= PrimitivePtr::new(Sphere::new());

    node.set_primitive(&sphere);

    Ok(SceneNodePtr::new(node))
}

pub fn lua_cube_constructor<'lua>(lua: Context<'lua>, lua_name: Value<'lua>) -> rlua::Result<SceneNodePtr> {
    let name = String::from_lua(lua_name, lua)?;

    let mut node = SceneNode::new(&name);
    let cube= PrimitivePtr::new(Cube::new());

    node.set_primitive(&cube);

    Ok(SceneNodePtr::new(node))
}

pub fn lua_mesh_constructor<'lua>(lua: Context<'lua>, lua_name: Value<'lua>, lua_file_name: Value<'lua>) -> rlua::Result<SceneNodePtr> {
    let name = String::from_lua(lua_name, lua)?;
    let file_name = String::from_lua(lua_file_name, lua)?;

    let mut node = SceneNode::new(&name);
    let mesh= PrimitivePtr::new(Mesh::new(&file_name));

    node.set_primitive(&mesh);

    Ok(SceneNodePtr::new(node))
}