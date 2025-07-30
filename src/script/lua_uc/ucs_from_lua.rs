use crate::script::LuaValueExt;
use crate::types::uc;
use mlua::FromLua;

impl FromLua for uc::UComp {
	fn from_lua(lua_value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
		// if not a table, assume Marker
		let mlua::Value::Table(table) = lua_value else {
			return Ok(uc::Marker::from_lua(lua_value, lua)?.into());
		};

		// NOTE: For now, only marker
		Ok(uc::Marker::from_lua(mlua::Value::Table(table), lua)?.into())
	}
}

// region:    --- Marker

const MARKER_DEFAULT_LABEL: &str = "Pin:";

impl FromLua for uc::Marker {
	fn from_lua(lua_value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
		if let mlua::Value::Table(table) = lua_value {
			let label = table.x_get_string("label").unwrap_or_else(|| MARKER_DEFAULT_LABEL.to_string());
			let content = table.x_get_string("content").unwrap_or_else(|| "No Content".to_string());
			Ok(uc::Marker { label, content })
		} else if let Some(content) = lua_value.as_string() {
			Ok(uc::Marker {
				label: MARKER_DEFAULT_LABEL.to_string(),
				content: content.to_string_lossy(),
			})
		} else {
			Err(mlua::Error::FromLuaConversionError {
				from: "Value",
				to: "PinCommand".to_string(),
				message: Some("expected a table with 'iden', 'priority', and 'content' keys".to_string()),
			})
		}
	}
}
// endregion: --- Marker
