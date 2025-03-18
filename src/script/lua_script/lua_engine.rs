use crate::Result;
use crate::hub::{HubEvent, get_hub};
use crate::runtime::RuntimeContext;
use crate::script::lua_script::helpers::{process_lua_eval_result, serde_value_to_lua_value};
use mlua::{IntoLua, Lua, Table, Value};

pub struct LuaEngine {
	lua: Lua,
	#[allow(unused)]
	runtime_context: RuntimeContext,
}

/// Constructors
impl LuaEngine {
	pub fn new(runtime_context: RuntimeContext) -> Result<Self> {
		let lua = Lua::new();

		// -- init utils (now under 'aip' namespace, and kept the 'utils')
		init_aip(&lua, &runtime_context)?;

		// -- backward compatibility "<=0.6.8"
		let globals = lua.globals();
		if let Ok(flow) = globals.get::<Table>("aip").and_then(|v| v.get::<Table>("flow")) {
			// TODO: will need to integrate trace or something like it
			let _ = globals.set("aipack", flow);
		}

		// -- init aipack (TODO: ths will need to be below the 'aip' namespace, once we find good submodule space)
		super::aip_flow::init_module(&lua, &runtime_context)?;

		// -- Init print
		init_print(&lua)?;

		// -- Build and return
		let engine = LuaEngine { lua, runtime_context };

		Ok(engine)
	}
}

/// Public Function
impl LuaEngine {
	pub fn eval(&self, script: &str, scope: Option<Table>, addl_lua_paths: Option<&[&str]>) -> Result<Value> {
		let lua = &self.lua;

		let chunck = lua.load(script);

		let chunck = if let Some(scope) = scope {
			let env = self.upgrade_scope(scope, addl_lua_paths)?;
			chunck.set_environment(env)
		} else {
			chunck
		};

		let res = chunck.eval::<Value>();
		// let res = res?;

		let res = process_lua_eval_result(&self.lua, res, script)?;

		Ok(res)
	}

	pub fn create_table(&self) -> Result<Table> {
		let res = self.lua.create_table()?;
		Ok(res)
	}

	/// Convert a json value to a lua value.
	///
	/// IMPORTANT: Use this to covert JSON Value to Lua Value, as the default mlua to_value,
	///            converts serde_json::Value::Null to Lua user data, and not mlua::Value::Nil,
	///            and we want it for aipack.
	pub fn serde_to_lua_value(&self, val: serde_json::Value) -> Result<Value> {
		serde_value_to_lua_value(&self.lua, val)
	}

	/// Just passthrough for into_lua
	#[allow(unused)]
	pub fn to_lua(&self, val: impl IntoLua) -> Result<Value> {
		let res = val.into_lua(&self.lua)?;
		Ok(res)
	}
}

/// private
impl LuaEngine {
	/// Upgrade a custom scope to full scope with all of the globals added.
	/// NOTE: A `base_lua_path` is the container of the `lua/` dir. So
	///       `base_lua_path = /some/dir`, the path added to lua package path will be `/some/dir/lua/?.lua,/some/dir/lua/?/init.lua`
	fn upgrade_scope(&self, scope: Table, addl_base_lua_paths: Option<&[&str]>) -> Result<Table> {
		// Get the globals table
		let globals = self.lua.globals();

		// Iterate over globals and add them to the scope table
		for pair in globals.pairs::<Value, Value>() {
			let (key, value) = pair?;
			scope.set(key, value)?; // Add each global to the scope table
		}

		// -- Prepend the additional lua path
		if let Some(addl_lua_paths) = addl_base_lua_paths {
			let mut paths: Vec<String> = Vec::new();
			for path in addl_lua_paths {
				paths.push(format!("{path}/lua/?.lua;{path}/lua/?/init.lua"));
			}
			if let Ok(lua_package) = globals.get::<Table>("package") {
				let path: String = lua_package.get("path")?;
				let new_path = format!("{};{path}", paths.join(";"));
				lua_package.set("path", new_path)?;
			}
		}

		// Return the updated scope table
		Ok(scope)
	}
}

// region:    --- Init Globals

fn init_print(lua: &Lua) -> Result<()> {
	let globals = lua.globals();

	globals.set(
		"print",
		lua.create_function(|_, args: mlua::Variadic<Value>| {
			let output: Vec<String> = args
				.iter()
				.map(|arg| match arg {
					Value::String(s) => s.to_str().map(|s| s.to_string()).unwrap_or_default(),
					Value::Number(n) => n.to_string(),
					Value::Integer(n) => n.to_string(),
					Value::Boolean(b) => b.to_string(),
					_ => "<unsupported value for print args>".to_string(),
				})
				.collect();

			let text = output.join("\t"); // Mimics Lua's `print` by joining args with tabs
			get_hub().publish_sync(HubEvent::LuaPrint(text.into()));
			Ok(())
		})?,
	)?;

	Ok(())
}

// endregion: --- Init Globals

// region:    --- init_utils

/// Just a convenient macro to init/set the lua modules
/// Will generate the code below for the name 'git'
/// ```rust
/// let git = utils_git::init_module(lua, runtime_context)?;
/// table.set("git", git)
/// ```
macro_rules! init_and_set {
    ($table:expr, $lua:expr, $runtime_context:expr, $($name:ident),*) => {
        paste::paste! {
            $(
                let $name = super::[<aip_ $name>]::init_module($lua, $runtime_context)?;
                $table.set(stringify!($name), $name)?;
            )*
        }
    };
}

/// Module builders
fn init_aip(lua_vm: &Lua, runtime_context: &RuntimeContext) -> Result<()> {
	// Note: using `lua_vm` to not conflict with the `lua` in the init_and_set that will get expanded as `lua` variable.
	let table = lua_vm.create_table()?;

	init_and_set!(
		table,
		lua_vm,
		runtime_context,
		// -- The lua module names that refers to utils_...
		flow,
		file,
		git,
		web,
		text,
		rust,
		path,
		md,
		json,
		html,
		cmd,
		lua,
		code,
		hbs,
		semver,
		agent
	);

	let globals = lua_vm.globals();
	// NOTE: now the aipack utilities are below `aip`,
	//       this way clearer that this does not belong to default lua.
	globals.set("aip", &table)?;
	// NOTE: For now, we keep the compatibility of utils.
	globals.set("utils", table)?;
	Ok(())
}

// endregion: --- init_utils

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::runtime::Runtime;

	/// Test if custom scope and global lua utils `math` work.
	#[tokio::test]
	async fn test_lua_engine_eval_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let engine = LuaEngine::new(runtime.context().clone())?;
		let fx_script = r#"
local square_root = math.sqrt(25)
return "Hello " .. my_name .. " - " .. square_root		
		"#;

		// -- Exec
		let scope = engine.create_table()?;
		scope.set("my_name", "Lua World")?;
		let res = engine.eval(fx_script, Some(scope), None)?;

		// -- Check
		let res = serde_json::to_value(res)?;
		let res = res.as_str().ok_or("Should be string")?;
		assert_eq!(res, "Hello Lua World - 5.0");
		Ok(())
	}
}

// endregion: --- Tests
