use std::sync::Arc;
use core::nodes::{Node, SceneNode};
use na::{Vector3};
use rlua::{UserData, UserDataMethods, Value};

pub struct LuaSceneNode {
    node: Arc<Node>
}

impl LuaSceneNode {
    pub fn new(node_name: &String) -> Self {
        LuaSceneNode { node: Arc::new(SceneNode::new(node_name)) }
    }
}

impl UserData for LuaSceneNode {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method_mut("rotate", |_, node, (lua_axis, lua_angle)| {
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

            let internal_node = &(node.node);

            (*internal_node).rotate(axis, angle);

            Ok(())
        });

        methods.add_method("scale", |_, node, (lua_x, lua_y, lua_z)| {
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
            let internal_node = &(node.node);

            (*internal_node).scale(&amount);

            Ok(())
        });

        methods.add_method("tranlate", |_, node, (lua_x, lua_y, lua_z)| {
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
            let internal_node = &(node.node);

            (*internal_node).translate(&amount);

            Ok(())
        });

        methods.add_method("add_child", |_, node, lua_node| {
            match lua_node {
                Value::UserData(user_data) => {
                    match user_data.borrow::<LuaSceneNode>() {
                        Ok(child) => {
                            let internal_node = &(node.node);
                            let internal_child = &(child.node);

                            (*internal_node).add_child(internal_child);

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
