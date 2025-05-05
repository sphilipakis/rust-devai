use crate::Result;
use mlua::{Lua, LuaSerdeExt as _};

/// Convert a json value to a lua value.
///
/// IMPORTANT: Use this to convert JSON Value to Lua Value, as the default mlua to_value,
///            converts serde_json::Value::Null to Lua user data, and not mlua::Value::Nil,
///            and we want it for aipack.
pub fn serde_value_to_lua_value(lua: &Lua, val: serde_json::Value) -> Result<mlua::Value> {
	let res = match val {
		serde_json::Value::Null => mlua::Value::Nil,
		other => lua.to_value(&other)?,
	};
	Ok(res)
}

/// Convenient function to take lua value to serde value
///
/// NOTE: The app should use this one rather to call serde_json::to_value directly
///       This way we can normalize the behavior and error and such.
pub fn lua_value_to_serde_value(lua_value: mlua::Value) -> Result<serde_json::Value> {
	let value = serde_json::to_value(lua_value)?;
	Ok(value)
}

/// Convert a vec of serde_json Value into a vec of Lua Value using serde_value_to_lua_value logic.
pub fn serde_values_to_lua_values(lua: &Lua, values: Vec<serde_json::Value>) -> Result<Vec<mlua::Value>> {
	values.into_iter().map(|val| serde_value_to_lua_value(lua, val)).collect()
}

