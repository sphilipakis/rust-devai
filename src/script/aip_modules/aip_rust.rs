//! Defines the `rust` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//! The `rust` module exposes functions used to process Rust code.
//!
//! ### Functions
//!
//! - `aip.rust.prune_to_declarations(code: string) -> string | {error: string}`

use crate::Result;
use crate::runtime::Runtime;
use crate::support::code::run_prune_to_declarations;
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let prune_fn = lua.create_function(prune_to_declarations)?;

	table.set("prune_to_declarations", prune_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Prunes Rust code to retain only function declarations.
///
/// ```lua
/// -- API Signature
/// aip.rust.prune_to_declarations(code: string): string | {error: string}
/// ```
///
/// Replaces function bodies with `{ ... }`, preserving comments, whitespace, and non-function code structures.
///
/// ### Arguments
///
/// - `code: string`: The Rust code to prune.
///
/// ### Returns
///
/// `string | {error: string}`: The pruned Rust code as a string on success, or a table with an error message on failure.
///
/// ### Error
///
/// Returns an error object (`{ error: string }`) if the pruning process fails.
///
/// ```ts
/// {
///   error : string         // Error message from rust code
/// }
/// ```
fn prune_to_declarations(lua: &Lua, code: String) -> mlua::Result<Value> {
	match run_prune_to_declarations(&code) {
		Ok(result) => Ok(Value::String(lua.create_string(&result)?)),
		Err(err) => Err(crate::Error::custom(format!("Failed to prune Rust code: {err}")).into()),
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules::aip_rust;

	#[tokio::test]
	async fn test_lua_rust_prune_to_declarations() -> Result<()> {
		// -- Fixtures
		let lua = setup_lua(aip_rust::init_module, "rust")?;
		let data_script = r#"
//! Some top comment 

use some::module; // and comment 

/// Some comment
pub fn async some_async_fn(some_arg: String) -> i32{
   let some = "code";
	 123
}

// Some fn normal
fn some_normal() {
		// DOING SOME STUFF
		// some fn stuff
}	 
		"#;
		// -- Exec
		let script = format!("return aip.rust.prune_to_declarations({data_script:?})");
		let res = eval_lua(&lua, &script)?;
		// -- Check
		let res = res.as_str().ok_or("Should be str")?;
		assert_contains(res, "use some::module; // and comment ");
		assert_contains(res, "async some_async_fn(some_arg: String) -> i32");
		assert_contains(res, "fn some_normal()");
		assert!(
			!res.contains(r#"let some = "code";"#),
			"should NOT contain let some ..."
		);
		assert!(!res.contains("// DOING SOME STUFF"), "DOING SOME STUFF");
		Ok(())
	}
}

// endregion: --- Tests
