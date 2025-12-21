//! Defines the `editor` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.editor` module exposes functions to interact with text editors.
//!
//! ### Functions
//!
//! - `aip.editor.open_file(path: string): EditorResult | nil`
//!   Opens a file in the auto-detected editor.
//!
//! ---
//!

use crate::Result;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::support::editor;
use mlua::{IntoLua, Lua, Table, Value};

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let rt = runtime.clone();
	let open_file_fn = lua.create_function(move |lua, path: String| open_file(lua, &rt, path))?;

	table.set("open_file", open_file_fn)?;

	Ok(table)
}

/// ## Lua Documentation
/// ---
/// Opens a file in the auto-detected editor.
///
/// ```lua
/// -- API Signature
/// aip.editor.open_file(path: string): EditorResult | nil
/// ```
///
/// Attempts to open the specified file in an auto-detected text editor.
/// The editor is detected based on environment variables in the following order:
/// 1. `ZED_TERM` environment variable (for Zed editor)
/// 2. `TERM_PROGRAM` environment variable (for various editors)
/// 3. `VISUAL` environment variable
/// 4. `EDITOR` environment variable
///
/// The path is resolved relative to the workspace root, similar to `aip.file.load`.
/// Pack references (e.g., `ns@pack/file.txt`) are supported.
///
/// ### Arguments
///
/// - `path: string` - The path to the file to open. Can be relative to workspace,
///   absolute, or a pack reference.
///
/// ### Returns
///
/// - `EditorResult | nil` - A table containing information about the editor used,
///   or `nil` if no editor could be detected.
///
/// The `EditorResult` table has the following structure:
/// ```lua
/// {
///   editor = string,  -- The name of the editor program (e.g., "zed", "code", "nvim")
/// }
/// ```
///
/// ### Example
///
/// ```lua
/// -- Open a file in the detected editor
/// local result = aip.editor.open_file("README.md")
/// if result then
///   print("Opened with:", result.editor)
/// else
///   print("No editor detected")
/// end
///
/// -- Open a file from a pack
/// aip.editor.open_file("ns@pack/main.aip")
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The path cannot be resolved (e.g., invalid pack reference).
/// - The file does not exist.
/// - The editor fails to launch.
///
/// ```ts
/// {
///   error: string  // Error message
/// }
/// ```
fn open_file(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();

	// Resolve the path similar to aip.file.load
	let full_path = dir_context.resolve_path(runtime.session(), (&path).into(), PathResolver::WksDir, None)?;

	// Check if file exists
	if !full_path.is_file() {
		return Err(crate::Error::custom(format!("File does not exist: {path}")).into());
	}

	// Attempt to open the file in the auto-detected editor
	match editor::open_file_auto(&full_path)? {
		Some(editor_program) => {
			let result = lua.create_table()?;
			result.set("editor", editor_program.program())?;
			result.into_lua(lua)
		}
		None => Ok(Value::Nil),
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_editor;

	#[tokio::test]
	async fn test_lua_editor_open_file_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_editor::init_module, "editor").await?;
		let code = r#"return aip.editor.open_file("non_existent_file.txt")"#;

		// -- Exec
		let res = eval_lua(&lua, code);

		// -- Check
		assert!(res.is_err(), "Should return error for non-existent file");
		let err = res.unwrap_err();
		assert!(
			err.to_string().contains("File does not exist"),
			"Error should mention file does not exist"
		);

		Ok(())
	}
}

// endregion: --- Tests
