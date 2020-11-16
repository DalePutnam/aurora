use na::Vector3;
use std::fmt;
use rlua::{Context, Value, Error, Result, FromLua};

pub fn from_lua<'lua, T>(lua_value: Value<'lua>, _lua: Context<'lua>) -> Result<Vector3<T>>
    where T: Copy + PartialEq + fmt::Debug + FromLua<'lua> + 'static {
    match lua_value {
        Value::Table(table) => {
            let table_length = table.len()?;

            if table_length != 3 {
                let msg = format!("Expected a table of length 3, found length {}", table_length);
                Err(Error::FromLuaConversionError {
                    from: "table",
                    to: "Vector3",
                    message: Some(msg)
                })
            } else {
                let x: T = table.get(1)?;
                let y: T = table.get(2)?;
                let z: T = table.get(3)?;

                Ok(Vector3::new(x, y, z))
            }
        },
        _ => {
            let msg = format!("Expected a table, found something else");
            Err(Error::FromLuaConversionError {
                from: "table",
                to: "Vector3",
                message: Some(msg)
            })
        },
    }
}
