use crate::script::LuaValueExt;
use mlua::{FromLua, Lua, Value};

#[derive(Debug, Default)]
pub struct FileOverOptions {
	/// If true (default), overwrite the destination if it exists.
	pub overwrite: Option<bool>,
}

impl FileOverOptions {
	pub fn overwrite(&self) -> bool {
		self.overwrite.unwrap_or(false)
	}
}

impl FromLua for FileOverOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		let table = value
			.as_table()
			.ok_or(crate::Error::custom("FileOverOptions should be a table"))?;

		let overwrite = table.x_get_bool("overwrite");

		Ok(Self { overwrite })
	}
}
