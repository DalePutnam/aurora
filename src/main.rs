extern crate rlua;
extern crate nalgebra as na;

mod core;
mod lua;

use lua::scene_lua::LuaSceneNode;
use rlua::{Lua, Value, Result};

fn main() {
    test_rlua().unwrap();
}

fn test_rlua() -> Result<()> {
    let lua = Lua::new();

    let globals = lua.globals();
    let gr = lua.create_table()?;

    let scene_node_ctor = lua.create_function(|_, lua_name: Value| {
        let name = match lua_name {
            Value::String(string) => string.to_str().unwrap().to_string(),
            _ => panic!("Failed to create node"),
        };

        Ok(LuaSceneNode::new(&name))
    })?;

    gr.set("node", scene_node_ctor)?;
    globals.set("gr", gr)?;

    lua.exec::<()>(
        r#"
            print("Hello World from Lua!")
            node = gr.node("Root")
            node2 = gr.node("Child")
            node:add_child(node2)

            node:rotate('X', 1.0)
            node:scale(1.0, 2.0, 1.0)
            node:tranlate(1.0, 1.0, 1.0)
        "#,
        Some("Test"),
    )?;

    Ok(())
}
