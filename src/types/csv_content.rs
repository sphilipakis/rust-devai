use crate::support::W;
use mlua::IntoLua;

pub struct CsvContent {
	pub headers: Vec<String>,
	pub rows: Vec<Vec<String>>,
}

// region:    --- Lua

impl IntoLua for CsvContent {
	/// Converts the `MdBlock` instance into a Lua Value
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("_type", "CsvContent")?;

		table.set("headers", W(self.headers).into_lua(lua)?)?;
		table.set("rows", W(self.rows).into_lua(lua)?)?;

		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
