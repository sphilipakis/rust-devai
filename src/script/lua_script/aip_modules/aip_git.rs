//! Defines the `git` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.git` module exposes functions for performing Git operations.
//!
//! ### Functions
//!
//! - `aip.git.restore(path: string): string`

use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::{Error, Result};
use mlua::{IntoLua, Lua, Table, Value};

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let rt = runtime.clone();
	let git_restore_fn = lua.create_function(move |lua, (path,): (String,)| git_restore(lua, &rt, path))?;

	table.set("restore", git_restore_fn)?;

	Ok(table)
}

// region: --- Lua Functions

/// ## Lua Documentation
///
/// Executes a `git restore` command in the workspace directory using the given file path.
///
/// ```lua
/// -- API Signature
/// aip.git.restore(path: string): string
/// ```
///
/// ### Arguments
///
/// - `path: string`: The file path to restore.
///
/// ### Returns
///
/// Returns the standard output as a string if the command is successful.
///
/// ### Example
///
/// ```lua
/// local result = aip.git.restore("src/main.rs")
/// print(result)
/// ```
///
/// ### Error
///
/// Throws an error if the command's stderr output is not empty.
fn git_restore(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let current_dir = runtime
		.dir_context()
		.try_wks_dir_with_err_ctx("aip.git.restore requires a aipack workspace setup")?;
	let output = std::process::Command::new("git")
		.current_dir(current_dir)
		.arg("restore")
		.arg(&path)
		.output()
		.expect("Failed to execute command");

	let stdout = String::from_utf8_lossy(&output.stdout);
	let stderr = String::from_utf8_lossy(&output.stderr);

	if !stderr.is_empty() {
		get_hub().publish_sync(format!("stderr: {}", stderr));
		return Err(Error::cc(format!("'git restore {path}' failed"), stderr).into());
	}

	stdout.into_lua(lua)
}

// endregion: --- Lua Functions
