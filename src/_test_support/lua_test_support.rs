use crate::Result;
use crate::runtime::Runtime;
use crate::script::process_lua_eval_result;
use mlua::{Lua, Table};
use serde_json::Value;

/// Sets up a Lua instance with both functions registered under `aip.` aip_name.
pub async fn setup_lua<F>(init_fn: F, sub_module: &str) -> Result<Lua>
where
	F: FnOnce(&Lua, &Runtime) -> Result<Table>,
{
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;

	let lua = Lua::new();
	let globals = lua.globals();
	let aip = lua.create_table()?;

	let path_table = init_fn(&lua, &runtime)?;
	// if sub_module is empty then, assume it is a table and set them one by one
	if sub_module.is_empty() {
		for pair in path_table.pairs::<String, mlua::Value>() {
			let (key, value) = pair?;
			aip.set(key, value)?;
		}
	}
	// otherwise add it in the sub module
	else {
		aip.set(sub_module, path_table)?;
	}

	globals.set("aip", &aip)?;
	// For backward compatiblity
	globals.set("utils", aip)?;

	Ok(lua)
}

pub fn eval_lua(lua: &Lua, code: &str) -> Result<Value> {
	let res = lua.load(code).eval::<mlua::Value>();
	let res_lua_value = process_lua_eval_result(lua, res, code)?;
	let value = serde_json::to_value(&res_lua_value)?;
	Ok(value)
}
