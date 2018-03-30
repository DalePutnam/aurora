use std::sync::{Arc, RwLock};
use core::nodes::SceneNode;
use na::Vector3;
use rlua::{UserData, UserDataMethods, Value};

pub struct LuaSceneNode {
    node: Arc<RwLock<SceneNode>>,
}

impl LuaSceneNode {
    pub fn new(node_name: &String) -> Self {
        LuaSceneNode { node: Arc::new(RwLock::new(SceneNode::new(node_name))) }
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

        methods.add_method("tranlate", |_, lua_node, (lua_x, lua_y, lua_z)| {
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
    }
}
