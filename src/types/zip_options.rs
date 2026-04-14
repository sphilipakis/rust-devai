use crate::script::LuaValueExt;
use mlua::{FromLua, Lua, Value};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ZipOptions {
	pub globs: Option<Vec<String>>,
}

impl ZipOptions {
	pub fn has_globs(&self) -> bool {
		self.globs.as_ref().is_some_and(|globs| !globs.is_empty())
	}
}

impl FromLua for ZipOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(Self::default()),
			Value::Table(table) => {
				let globs = if let Some(val) = table.x_get_value("globs") {
					match val {
						Value::Table(t) => {
							let mut globs = Vec::new();
							for value in t.sequence_values::<String>() {
								let value = value.map_err(|e| mlua::Error::FromLuaConversionError {
									from: "Table",
									to: "Vec<String>".to_string(),
									message: Some(format!("Invalid globs: {e}")),
								})?;
								globs.push(value);
							}
							Some(globs)
						}
						other => {
							return Err(mlua::Error::FromLuaConversionError {
								from: other.type_name(),
								to: "Vec<String>".to_string(),
								message: Some("ZipOptions.globs must be an array of strings".into()),
							});
						}
					}
				} else {
					None
				};

				Ok(Self { globs })
			}
			other => Err(mlua::Error::FromLuaConversionError {
				from: other.type_name(),
				to: "ZipOptions".to_string(),
				message: Some("Expected nil or a table for ZipOptions".into()),
			}),
		}
	}
}
