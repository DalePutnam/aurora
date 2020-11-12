use rlua::{Context, FromLua, Result, Error, Value};
use std::fmt;
use na::Vector3;

pub struct LuaVector3<T>
    where T: Copy + PartialEq + fmt::Debug + 'static
{
    vector: Vector3<T>,
}

impl<T> LuaVector3<T>
    where T: Copy + PartialEq + fmt::Debug + 'static
{
    pub fn new(x: T, y: T, z: T) -> Self {
        LuaVector3 {
            vector: Vector3::new(x, y, z),
        }
    }

    pub fn get_inner(&self) -> Vector3<T> {
        self.vector
    }
}

impl<'lua, T> FromLua<'lua> for LuaVector3<T>
    where T: Copy + PartialEq + fmt::Debug + FromLua<'lua>
{
    fn from_lua(lua_value: Value<'lua>, _lua: Context<'lua>) -> Result<Self> {
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

                    Ok(LuaVector3::new(x, y, z))
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
}