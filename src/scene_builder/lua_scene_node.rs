use std::cell::RefCell;
use std::sync::{RwLock, Arc};
use na::{Matrix4, Vector3, Unit};
use rlua;
use rlua::{UserData, UserDataMethods, Value};
use core::traits::Primitive;
use core::{NonhierBox, NonhierSphere, Sphere, Cube, Mesh, Material, Object};
use scene_builder::lua_material::LuaMaterial;

pub struct LuaSceneNode {
    node: Arc<RwLock<SceneNode>>,
}

impl LuaSceneNode {
    fn new(scene_node: SceneNode) -> Self {
        LuaSceneNode { node: Arc::new(RwLock::new(scene_node)) }
    }

    pub (crate) fn convert_to_object_list(&self) -> Vec<Arc<Object>> {
        let list = RefCell::new(Vec::new());
        let mut id = 0;

        match self.node.read() {
            Ok(scene_node) => {
                scene_node.convert_to_object_list(&list, &Matrix4::identity(), &mut id);
            },
            Err(_) => panic!("Node lock is poisoned!"),
        };

        list.into_inner()
    }
}

impl UserData for LuaSceneNode {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method_mut("rotate", |_, lua_node, (lua_axis, lua_angle)| {
            let axis = match lua_axis {
                Value::String(string) => {
                    match string.to_str() {
                        Ok(slice) => slice.chars().nth(0).unwrap(),
                        Err(e) => panic!(e),
                    }
                },
                _ => panic!("Failed to rotate"),
            };

            let angle = match lua_angle {
                Value::Number(number) => number as f32,
                Value::Integer(integer) => integer as f32,
                _ => panic!("Failed to rotate"),
            };

            match lua_node.node.write() {
                Ok(mut scene_node) => scene_node.rotate(axis, angle),
                Err(_) => panic!("SceneNode lock is poisoned!"),
            };

            Ok(())
        });

        methods.add_method("scale", |_, lua_node, (lua_x, lua_y, lua_z)| {
            let x = match lua_x {
                Value::Number(number) => number as f32,
                Value::Integer(integer) => integer as f32,
                _ => panic!("Failed to scale"),
            };

            let y = match lua_y {
                Value::Number(number) => number as f32,
                Value::Integer(integer) => integer as f32,
                _ => panic!("Failed to scale"),
            };

            let z = match lua_z {
                Value::Number(number) => number as f32,
                Value::Integer(integer) => integer as f32,
                _ => panic!("Failed to scale"),
            };

            let amount = Vector3::new(x, y, z);

            match lua_node.node.write() {
                Ok(mut scene_node) => scene_node.scale(&amount),
                Err(_) => panic!("SceneNode lock is poisoned!"),
            };

            Ok(())
        });

        methods.add_method("translate", |_, lua_node, (lua_x, lua_y, lua_z)| {
            let x = match lua_x {
                Value::Number(number) => number as f32,
                Value::Integer(integer) => integer as f32,
                _ => panic!("Failed to scale"),
            };

            let y = match lua_y {
                Value::Number(number) => number as f32,
                Value::Integer(integer) => integer as f32,
                _ => panic!("Failed to scale"),
            };

            let z = match lua_z {
                Value::Number(number) => number as f32,
                Value::Integer(integer) => integer as f32,
                _ => panic!("Failed to scale"),
            };

            let amount = Vector3::new(x, y, z);
            match lua_node.node.write() {
                Ok(mut scene_node) => scene_node.translate(&amount),
                Err(_) => panic!("SceneNode lock is poisoned!"),
            };

            Ok(())
        });

        methods.add_method("add_child", |_, lua_node, child_lua_node| {
            match child_lua_node {
                Value::UserData(user_data) => {
                    match user_data.borrow::<LuaSceneNode>() {
                        Ok(child_node) => {
                            match lua_node.node.write() {
                                Ok(mut scene_node) => scene_node.add_child(&child_node.node),
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
                    match user_data.borrow::<LuaMaterial>() {
                        Ok(material) => {
                            match lua_node.node.write() {
                                Ok(mut scene_node) => scene_node.set_material(material.get_internal_material()),
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

pub fn lua_node_constructor(lua_name: Value) -> rlua::Result<LuaSceneNode> {
    let name = match lua_name {
        Value::String(string) => string.to_str().unwrap().to_string(),
        _ => panic!("Failed to create node"),
    };

    Ok(LuaSceneNode::new(SceneNode::new(&name)))
}

pub fn lua_nh_sphere_constructor(lua_name: Value, lua_position: Value, lua_radius: Value) -> rlua::Result<LuaSceneNode> {
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

pub fn lua_nh_box_constructor(lua_name: Value, lua_position: Value, lua_size: Value) -> rlua::Result<LuaSceneNode> {
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

pub fn lua_sphere_constructor(lua_name: Value) -> rlua::Result<LuaSceneNode> {
    let name = match lua_name {
        Value::String(string) => string.to_str().unwrap().to_string(),
        _ => return Err(rlua::Error::RuntimeError("Failed to create sphere".to_string())),
    };

    let mut node = SceneNode::new(&name);
    let sphere: Arc<Box<Primitive>> = Arc::new(Box::new(Sphere::new()));

    node.set_primitive(&sphere);

    Ok(LuaSceneNode::new(node))
}

pub fn lua_cube_constructor(lua_name: Value) -> rlua::Result<LuaSceneNode> {
    let name = match lua_name {
        Value::String(string) => string.to_str().unwrap().to_string(),
        _ => return Err(rlua::Error::RuntimeError("Failed to create cube".to_string())),
    };

    let mut node = SceneNode::new(&name);
    let cube: Arc<Box<Primitive>> = Arc::new(Box::new(Cube::new()));

    node.set_primitive(&cube);

    Ok(LuaSceneNode::new(node))
}

pub fn lua_mesh_constructor(lua_name: Value, lua_file_name: Value) -> rlua::Result<LuaSceneNode> {
    let name = match lua_name {
        Value::String(string) => string.to_str().unwrap().to_string(),
        _ => return Err(rlua::Error::RuntimeError("Failed to create mesh".to_string())),
    };

    let file_name = match lua_file_name {
        Value::String(string) => string.to_str().unwrap().to_string(),
        _ => return Err(rlua::Error::RuntimeError("Failed to create mesh".to_string())),
    };

    let mut node = SceneNode::new(&name);
    let mesh: Arc<Box<Primitive>> = Arc::new(Box::new(Mesh::new(&file_name)));

    node.set_primitive(&mesh);

    Ok(LuaSceneNode::new(node))
}

struct SceneNode {
    _name: String,
    transform: Matrix4<f32>,
    children: Vec<Arc<RwLock<SceneNode>>>,
    primitive: Option<Arc<Box<Primitive>>>,
    material: Option<Arc<Box<Material>>>
}

impl SceneNode {
    fn new(name: &str) -> Self {
        SceneNode { 
            _name: name.to_string(),
            transform: Matrix4::identity(),
            children: Vec::new(),
            primitive: None,
            material: None,
        }
    }

    fn convert_to_object_list(&self, list: &RefCell<Vec<Arc<Object>>>, transform: &Matrix4<f32>, current_id: &mut u64) {
        let new_transform = transform * self.transform;

        if self.primitive.is_some() && self.material.is_some() {
            let primitive = Arc::clone(self.primitive.as_ref().unwrap());
            let material = Arc::clone(self.material.as_ref().unwrap());
            let mut list_mut = list.borrow_mut();

            list_mut.push(Arc::new(Object::new(*current_id, &new_transform, &primitive, &material)));
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

    fn set_primitive(&mut self, primitive: &Arc<Box<Primitive>>) {
        self.primitive = Some(Arc::clone(primitive));
    }

    fn set_material(&mut self, material: &Arc<Box<Material>>) {
        self.material = Some(Arc::clone(material));
    }

    fn rotate(&mut self, axis: char, angle: f32) {
        let rotation_axis = match axis {
            'x' | 'X' => Vector3::new(1.0, 0.0, 0.0),
            'y' | 'Y' => Vector3::new(0.0, 1.0, 0.0),
            'z' | 'Z' => Vector3::new(0.0, 0.0, 1.0),
             _  => return,
        };

        let rotation_matrix = Matrix4::from_axis_angle(&Unit::new_normalize(rotation_axis), angle.to_radians());
        self.transform = rotation_matrix * self.transform;
    }

    fn scale(&mut self, amount: &Vector3<f32>) {
        let scale_matrix = Matrix4::new_nonuniform_scaling(amount);
        self.transform = scale_matrix * self.transform;
    }

    fn translate(&mut self, amount: &Vector3<f32>) {
        let translate_matrix = Matrix4::new_translation(amount);
        self.transform = translate_matrix * self.transform;
    }

    fn add_child(&mut self, node: &Arc<RwLock<SceneNode>>) {
        self.children.push(Arc::clone(node));
    }
}

