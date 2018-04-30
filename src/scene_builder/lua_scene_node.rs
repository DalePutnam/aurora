use std::sync::{RwLock, Arc};
use na::{Matrix4, Vector3, Unit, Vector4};
use rlua;
use rlua::{UserData, UserDataMethods, Value};
use core::math::degrees_to_radians;
use core::Material;
use core::traits::Primitive;
use scene_builder::lua_material::LuaMaterial;

pub struct LuaSceneNode {
    node: Arc<RwLock<SceneNode>>,
}

impl LuaSceneNode {
    pub fn new(scene_node: SceneNode) -> Self {
        LuaSceneNode { node: Arc::new(RwLock::new(scene_node)) }
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

pub struct SceneNode {
    name: String,
    transform_matrix: Matrix4<f32>,
    inverse_matrix: Matrix4<f32>,
    children: Vec<Arc<RwLock<SceneNode>>>,
    primitive: Option<Arc<Box<Primitive>>>,
    material: Option<Arc<Material>>
}

impl SceneNode {
    pub fn new(name: &str) -> Self {
        SceneNode { 
            name: name.to_string(),
            transform_matrix: Matrix4::<f32>::identity(),
            inverse_matrix: Matrix4::<f32>::identity(),
            children: Vec::new(),
            primitive: None,
            material: None,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_primitive(&self) -> &Option<Arc<Box<Primitive>>> {
        &self.primitive
    }

    pub fn set_primitive(&mut self, primitive: &Arc<Box<Primitive>>) {
        self.primitive = Some(Arc::clone(primitive));
    }

    pub fn get_material(&self) -> &Option<Arc<Material>> {
        &self.material
    }

    pub fn set_material(&mut self, material: &Arc<Material>) {
        self.material = Some(Arc::clone(material));
    }

    pub fn get_transform(&self) -> &Matrix4<f32> {
        &self.transform_matrix
    }

    pub fn get_inverse_transform(&self) -> &Matrix4<f32> {
        &self.inverse_matrix
    }

    pub fn rotate(&mut self, axis: char, angle: f32) {
        let rotation_axis = match axis {
            'x' | 'X' => Vector3::<f32>::new(1.0, 0.0, 0.0),
            'y' | 'Y' => Vector3::<f32>::new(0.0, 1.0, 0.0),
            'z' | 'Z' => Vector3::<f32>::new(0.0, 0.0, 1.0),
             _  => return,
        };

        let rotation_matrix = Matrix4::from_axis_angle(&Unit::new_normalize(rotation_axis), degrees_to_radians(angle));
        let new_matrix = rotation_matrix * self.transform_matrix;

        set_transform(&new_matrix, &mut self.transform_matrix, &mut self.inverse_matrix);
    }

    pub fn scale(&mut self, amount: &Vector3<f32>) {
        let scale_matrix = Matrix4::new_nonuniform_scaling(amount);
        let new_matrix = scale_matrix * self.transform_matrix;

        set_transform(&new_matrix, &mut self.transform_matrix, &mut self.inverse_matrix);
    }

    pub fn translate(&mut self, amount: &Vector3<f32>) {
        let translate_matrix = Matrix4::new_translation(amount);
        let new_matrix = translate_matrix * self.transform_matrix;

        set_transform(&new_matrix, &mut self.transform_matrix, &mut self.inverse_matrix);
    }

    pub fn add_child(&mut self, node: &Arc<RwLock<SceneNode>>) {
        self.children.push(Arc::clone(node));
    }
}

fn set_transform(transform: &Matrix4<f32>, matrix: &mut Matrix4<f32>, inverse_matrix: &mut Matrix4<f32>) {
    *matrix = *transform;
    *inverse_matrix = matrix.try_inverse().unwrap();
}
