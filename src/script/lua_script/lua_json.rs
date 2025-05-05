use crate::{Error, Result};
use mlua::{Lua, LuaSerdeExt as _};

// region:    --- Json To Lua

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

/// Convert a vec of serde_json Value into a vec of Lua Value using serde_value_to_lua_value logic.
pub fn serde_values_to_lua_values(lua: &Lua, values: Vec<serde_json::Value>) -> Result<Vec<mlua::Value>> {
	values.into_iter().map(|val| serde_value_to_lua_value(lua, val)).collect()
}

// endregion: --- Json To Lua

// region:    --- Lua To Json

/// Convenient function to take lua value to serde value
///
/// NOTE: The app should use this one rather to call serde_json::to_value directly
///       This way we can normalize the behavior and error and such.
pub fn lua_value_to_serde_value(lua_value: mlua::Value) -> Result<serde_json::Value> {
	let value = serde_json::to_value(lua_value)?;
	Ok(value)
}

/// Convert a lua Value that should be a Table/List to a Vec of serde values
pub fn lua_value_list_to_serde_values(lua_value: mlua::Value) -> Result<Vec<serde_json::Value>> {
	let list = match lua_value {
		mlua::Value::Table(table) => table,
		other => {
			return Err(Error::custom(format!(
				"Lua Value is not a List. Expected a Lua table (list) as the second argument, but got {}",
				other.type_name()
			)));
		}
	};

	let iter = list.sequence_values::<mlua::Value>();

	let json_values: Vec<serde_json::Value> = iter
		.into_iter()
		.map(|val| lua_value_to_serde_value(val?))
		.collect::<Result<Vec<_>>>()
		.map_err(|e| format!("A mlua Value cannot be serialize to Json. Cause: {e}"))?;

	Ok(json_values)
}

// endregion: --- Lua To Json
