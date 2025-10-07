use crate::script::LuaValueExt;
use mlua::{FromLua, Lua, Value};

/// Options to adjust how `aip.file.save` processes content before persisting it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SaveOptions {
	pub trim_start: Option<bool>,
	pub trim_end: Option<bool>,
	pub single_trailing_newline: Option<bool>,
}

impl SaveOptions {
	pub fn should_trim_start(&self) -> bool {
		self.trim_start.unwrap_or(false)
	}

	pub fn should_trim_end(&self) -> bool {
		self.trim_end.unwrap_or(false)
	}

	pub fn should_single_trailing_newline(&self) -> bool {
		self.single_trailing_newline.unwrap_or(false)
	}

	pub fn is_empty(&self) -> bool {
		self.trim_start.is_none() && self.trim_end.is_none() && self.single_trailing_newline.is_none()
	}
}

impl FromLua for SaveOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(Self::default()),
			Value::Table(table) => {
				let trim_start = table.x_get_bool("trim_start");
				let trim_end = table.x_get_bool("trim_end");
				let single_trailing_newline = table.x_get_bool("single_trailing_newline");

				Ok(Self {
					trim_start,
					trim_end,
					single_trailing_newline,
				})
			}
			other => Err(mlua::Error::FromLuaConversionError {
				from: other.type_name(),
				to: "SaveOptions".to_string(),
				message: Some(
					"SaveOptions argument can be nil or a table { trim_start?, trim_end?, single_trailing_newline? }"
						.into(),
				),
			}),
		}
	}
}
