//! Defines the `path` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//! The `aip.path` module exposes functions used to interact with file paths.
//!
//! ### Functions
//!
//! - `aip.path.split(path: string): parent: string, filename: string`
//! - `aip.path.resolve(path: string): string`
//! - `aip.path.exists(path: string): boolean`
//! - `aip.path.is_file(path: string): boolean`
//! - `aip.path.is_dir(path: string): boolean`
//! - `aip.path.diff(file_path: string, base_path: string): string`
//! - `aip.path.parent(path: string): string | nil`
//! - `aip.path.join(base: string, ...parts: string | string[]): string`
//! - `aip.path.parse(path: string | nil): table | nil`
//!

use crate::Result;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::support::{into_option_string, into_vec_of_strings};
use crate::types::FileInfo;
use mlua::{IntoLua, Lua, MultiValue, Table, Value, Variadic};
use simple_fs::SPath;
use std::path::Path;

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	// -- split
	let path_split_fn = lua.create_function(path_split)?;

	// -- exists
	let rt = runtime.clone();
	let path_exists_fn = lua.create_function(move |_lua, path: String| path_exists(&rt, path))?;

	// -- resolve
	let rt = runtime.clone();
	let path_resolve_fn = lua.create_function(move |_lua, path: String| path_resolve(&rt, path))?;

	// -- is_file
	let rt = runtime.clone();
	let path_is_file_fn = lua.create_function(move |_lua, path: String| path_is_file(&rt, path))?;

	// -- is_dir
	let rt = runtime.clone();
	let path_is_dir_fn = lua.create_function(move |_lua, path: String| path_is_dir(&rt, path))?;

	// -- diff
	let path_diff_fn =
		lua.create_function(move |_lua, (file_path, base_path): (String, String)| path_diff(file_path, base_path))?;

	// -- join
	let path_join =
		lua.create_function(move |lua, (base, args): (String, Variadic<Value>)| path_join(lua, base, args))?;

	// -- parse
	let path_parse_fn = lua.create_function(path_parse)?;

	// -- parent
	let path_parent_fn = lua.create_function(move |_lua, path: String| path_parent(path))?;

	// -- Add all functions to the module
	table.set("parse", path_parse_fn)?;
	table.set("resolve", path_resolve_fn)?;
	table.set("join", path_join)?;
	table.set("exists", path_exists_fn)?;
	table.set("is_file", path_is_file_fn)?;
	table.set("is_dir", path_is_dir_fn)?;
	table.set("diff", path_diff_fn)?;
	table.set("parent", path_parent_fn)?;
	table.set("split", path_split_fn)?;

	Ok(table)
}

// region:    --- Lua Functions

/// ## Lua Documentation
///
/// Parses a path string and returns a [FileInfo](#filemeta) table representation of its components.
///
/// ```lua
/// -- API Signature
/// aip.path.parse(path: string | nil): table | nil
/// ```
///
/// Parses the given path string into a structured table containing components like `dir`, `name`, `stem`, `ext`, etc., without checking file existence or metadata.
///
/// ### Arguments
///
/// - `path: string | nil`: The path string to parse. If `nil`, the function returns `nil`.
///
/// ### Returns
///
/// - `table | nil`: A [FileInfo](#filemeta) table representing the parsed path components if `path` is a string. Returns `nil` if the input `path` was `nil`. Note that `created_epoch_us`, `modified_epoch_us`, and `size` fields will be `nil` as this function only parses the string, it does not access the filesystem.
///
/// ### Example
///
/// ```lua
/// local parsed = aip.path.parse("some/folder/file.txt")
/// -- parsed will be similar to { path = "some/folder/file.txt", dir = "some/folder", name = "file.txt", stem = "file", ext = "txt", created_epoch_us = nil, ... }
/// print(parsed.name) -- Output: "file.txt"
///
/// local nil_result = aip.path.parse(nil)
/// -- nil_result will be nil
/// ```
///
/// ### Error
///
/// Returns an error (Lua table `{ error: string }`) if the path string is provided but is invalid and cannot be parsed into a valid SPath object.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_parse(lua: &Lua, path: Value) -> mlua::Result<Value> {
	let Some(path) = into_option_string(path, "aip.path.parse")? else {
		return Ok(Value::Nil);
	};

	let spath = SPath::new(path);
	let meta = FileInfo::new(spath, false);
	meta.into_lua(lua)
}

/// ## Lua Documentation
///
/// Split path into parent, filename.
///
/// ```lua
/// -- API Signature
/// aip.path.split(path: string): parent: string, filename: string
/// ```
///
/// Splits the given path into its parent directory component and its filename component.
///
/// ### Arguments
///
/// - `path: string`: The path to split.
///
/// ### Returns
///
/// Returns two strings: the parent directory path and the filename. Returns empty strings
/// if the path has no parent or no filename, respectively (e.g., splitting "." returns "", ".").
///
/// ```lua
/// -- Example output
/// local parent, filename = "some/path", "file.txt"
/// ```
///
/// ### Example
///
/// ```lua
/// local parent, filename = aip.path.split("folder/file.txt")
/// print(parent)   -- Output: "folder"
/// print(filename) -- Output: "file.txt"
///
/// local parent, filename = aip.path.split("justafile.md")
/// print(parent)   -- Output: ""
/// print(filename) -- Output: "justafile.md"
/// ```
///
/// ### Error
///
/// This function does not typically error, returning empty strings for components that do not exist.
fn path_split(lua: &Lua, path: String) -> mlua::Result<MultiValue> {
	let path = SPath::from(path);

	let parent = path.parent().map(|p| p.to_string()).unwrap_or_default();
	let file_name = path.file_name().unwrap_or_default().to_string();

	Ok(MultiValue::from_vec(vec![
		mlua::Value::String(lua.create_string(parent)?),
		mlua::Value::String(lua.create_string(file_name)?),
	]))
}

/// ## Lua Documentation
///
/// Joins a base path with one or more path segments.
///
/// ```lua
/// -- API Signature
/// aip.path.join(base: string, ...parts: string | string[]): string
/// ```
///
/// Constructs a new path by appending processed segments from `...parts` to the `base` path.
/// Each argument in `...parts` is first converted to a string:
/// - String arguments are used as-is.
/// - List (table) arguments have their string items joined by `/`.
///   These resulting strings are then concatenated together. Finally, this single concatenated string
///   is joined with `base` using system-appropriate path logic (which also normalizes separators like `//` to `/`).
///
/// ### Arguments
///
/// - `base: string`: The initial base path.
/// - `...parts: string | string[]` (variadic): One or more path segments to process and append.
///   Each part can be a single string or a Lua list (table) of strings.
///
/// ### Returns
///
/// - `string`: A new string representing the combined and normalized path.
///
/// ### Example
///
/// ```lua
/// -- Example 1: Basic join
/// print(aip.path.join("dir1/", "file1.txt"))             -- Output: "dir1/file1.txt"
/// print(aip.path.join("dir1", "file1.txt"))              -- Output: "dir1/file1.txt"
///
/// -- Example 2: Joining with a list (table)
/// print(aip.path.join("dir1/", {"subdir", "file2.txt"})) -- Output: "dir1/subdir/file2.txt"
///
/// -- Example 3: Multiple string arguments
/// -- Segments are concatenated, then joined to base.
/// print(aip.path.join("dir1/", "subdir/", "file3.txt"))  -- Output: "dir1/subdir/file3.txt"
/// print(aip.path.join("dir1/", "subdir", "file3.txt"))   -- Output: "dir1/subdirfile3.txt"
///
/// -- Example 4: Mixed arguments (strings and lists)
/// -- Lists are pre-joined with '/', then all resulting strings are concatenated, then joined to base.
/// print(aip.path.join("root/", {"user", "docs"}, "projectA", {"report", "final.pdf"}))
/// -- Output: "root/user/docsprojectAreport/final.pdf"
///
/// -- Example 5: Normalization
/// print(aip.path.join("", {"my-dir//", "///file.txt"}))  -- Output: "my-dir/file.txt"
/// print(aip.path.join("a", "b", "c"))                     -- Output: "a/bc"
/// print(aip.path.join("a/", "b/", "c/"))                  -- Output: "a/b/c/"
/// ```
///
/// ### Error
///
/// Returns an error (Lua table `{ error: string }`) if any of the `parts` arguments cannot be
/// converted to a string or a list of strings (e.g., passing a boolean or a function).
fn path_join(lua: &Lua, base: String, parts: Variadic<Value>) -> mlua::Result<Value> {
	let base = SPath::from(base);

	let mut parts_str = String::new();
	for part in parts {
		// TODO: Could optimize a little to not put a Vec when single value
		let sub_parts = into_vec_of_strings(part, "aip.path.join")?;
		parts_str.push_str(&sub_parts.join("/"))
	}

	let res = base.join(parts_str).to_string();
	let res = res.into_lua(lua)?;
	Ok(res)
}

/// ## Lua Documentation
///
/// Resolves and normalizes a path relative to the workspace.
///
/// ```lua
/// -- API Signature
/// aip.path.resolve(path: string): string
/// ```
///
/// Resolves and normalizes the given path string. This handles relative paths (`.`, `..`),
/// absolute paths, and special aipack pack references (`ns@pack/`, `ns@pack$base/`, `ns@pack$workspace/`).
/// The resulting path is normalized (e.g., `some/../path` becomes `path`). Note: `some/path` was a typo in original, fixed to `path`.
///
/// ### Arguments
///
/// - `path: string`: The path string to resolve and normalize. It can be a relative path, an absolute path, or an aipack pack reference.
///
/// ### Returns
///
/// - `string`: The resolved and normalized path as a string. This path is usually absolute after resolution.
///
/// ### Example
///
/// ```lua
/// local resolved_path = aip.path.resolve("./agent-script/../agent-script/agent-hello.aip")
/// print(resolved_path) -- Output: "/path/to/workspace/agent-script/agent-hello.aip" (example)
///
/// local resolved_pack_path = aip.path.resolve("ns@pack/some/file.txt")
/// print(resolved_pack_path) -- Output: "/path/to/aipack-base/packs/ns/pack/some/file.txt" (example)
/// ```
///
/// ### Error
///
/// Returns an error if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_resolve(runtime: &Runtime, path: String) -> mlua::Result<String> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&path).into(), PathResolver::WksDir)?;
	Ok(path.to_string())
}

/// ## Lua Documentation
///
/// Checks if the specified path exists.
///
/// ```lua
/// -- API Signature
/// aip.path.exists(path: string): boolean
/// ```
///
/// Checks if the file or directory specified by `path` exists. The path is resolved relative to the workspace root.
/// Supports relative paths, absolute paths, and pack references (`ns@pack/...`).
///
/// ### Arguments
///
/// - `path: string`: The path string to check for existence. Can be relative, absolute, or a pack reference.
///
/// ### Returns
///
/// - `boolean`: Returns `true` if a file or directory exists at the specified path, `false` otherwise.
///
/// ### Example
///
/// ```lua
/// if aip.path.exists("README.md") then
///   print("README.md exists.")
/// end
///
/// if aip.path.exists("ns@pack/main.aip") then
///   print("Pack main agent exists.")
/// end
/// ```
///
/// ### Error
///
/// Returns an error if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_exists(runtime: &Runtime, path: String) -> mlua::Result<bool> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&path).into(), PathResolver::WksDir)?;
	Ok(path.exists())
}

/// ## Lua Documentation
///
/// Checks if the specified path points to a file.
///
/// ```lua
/// -- API Signature
/// aip.path.is_file(path: string): boolean
/// ```
///
/// Checks if the entity at the specified `path` is a file. The path is resolved relative to the workspace root.
/// Supports relative paths, absolute paths, and pack references (`ns@pack/...`).
///
/// ### Arguments
///
/// - `path: string`: The path string to check. Can be relative, absolute, or a pack reference.
///
/// ### Returns
///
/// - `boolean`: Returns `true` if the path points to an existing file, `false` otherwise.
///
/// ### Example
///
/// ```lua
/// if aip.path.is_file("config.toml") then
///   print("config.toml is a file.")
/// end
///
/// if aip.path.is_file("src/") then
///   print("src/ is a file.") -- This will print false
/// end
/// ```
///
/// ### Error
///
/// Returns an error if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_is_file(runtime: &Runtime, path: String) -> mlua::Result<bool> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&path).into(), PathResolver::WksDir)?;
	Ok(path.is_file())
}

/// ## Lua Documentation
///
/// Computes the relative path from `base_path` to `file_path`.
///
/// ```lua
/// -- API Signature
/// aip.path.diff(file_path: string, base_path: string): string
/// ```
///
/// Calculates the relative path string that navigates from the `base_path` to the `file_path`.
/// Both paths can be absolute or relative.
///
/// ### Arguments
///
/// - `file_path: string`: The target path.
/// - `base_path: string`: The starting path.
///
/// ### Returns
///
/// - `string`: The relative path string from `base_path` to `file_path`. Returns an empty string if the paths are the same or if a relative path cannot be easily computed (e.g., on different drives on Windows).
///
/// ### Example
///
/// ```lua
/// print(aip.path.diff("/a/b/c/file.txt", "/a/b/")) -- Output: "c/file.txt"
/// print(aip.path.diff("/a/b/", "/a/b/c/file.txt")) -- Output: "../.."
/// print(aip.path.diff("/a/b/c", "/a/d/e"))      -- Output: "../../b/c" (example, depends on OS)
/// print(aip.path.diff("folder/file.txt", "folder")) -- Output: "file.txt"
/// print(aip.path.diff("folder/file.txt", "folder/file.txt")) -- Output: ""
/// ```
///
/// ### Error
///
/// Returns an error if the paths are invalid or cannot be processed.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_diff(file_path: String, base_path: String) -> mlua::Result<String> {
	let file_path = SPath::from(file_path);
	let base_path = SPath::from(base_path);
	// NOTE: Right now, using unwrap_or_default, as this should not happen
	//       But will update simple-fs to utf8 diff by default
	let diff = file_path.diff(base_path).map(|p| p.to_string()).unwrap_or_default();
	Ok(diff)
}

/// ## Lua Documentation
///
/// Checks if the specified path points to a directory.
///
/// ```lua
/// -- API Signature
/// aip.path.is_dir(path: string): boolean
/// ```
///
/// Checks if the entity at the specified `path` is a directory. The path is resolved relative to the workspace root.
/// Supports relative paths, absolute paths, and pack references (`ns@pack/...`).
///
/// ### Arguments
///
/// - `path: string`: The path string to check. Can be relative, absolute, or a pack reference.
///
/// ### Returns
///
/// - `boolean`: Returns `true` if the path points to an existing directory, `false` otherwise.
///
/// ### Example
///
/// ```lua
/// if aip.path.is_dir("src/") then
///   print("src/ is a directory.")
/// end
///
/// if aip.path.is_dir("config.toml") then
///   print("config.toml is a directory.") -- This will print false
/// end
/// ```
///
/// ### Error
///
/// Returns an error if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_is_dir(runtime: &Runtime, path: String) -> mlua::Result<bool> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&path).into(), PathResolver::WksDir)?;
	Ok(path.is_dir())
}

/// ## Lua Documentation
///
/// Returns the parent directory path of the specified path.
///
/// ```lua
/// -- API Signature
/// aip.path.parent(path: string): string | nil
/// ```
///
/// Gets the parent directory component of the given path string.
///
/// ### Arguments
///
/// - `path: string`: The path string.
///
/// ### Returns
///
/// - `string | nil`: Returns the parent directory path as a string. Returns `nil` if the path has no parent (e.g., ".", "/", "C:/").
///
/// ### Example
///
/// ```lua
/// print(aip.path.parent("some/path/file.txt")) -- Output: "some/path"
/// print(aip.path.parent("/root/file"))         -- Output: "/root"
/// print(aip.path.parent("."))                  -- Output: nil
/// ```
///
/// ### Error
///
/// This function does not typically error.
fn path_parent(path: String) -> mlua::Result<Option<String>> {
	match Path::new(&path).parent() {
		Some(parent) => match parent.to_str() {
			Some(parent_str) => Ok(Some(parent_str.to_string())),
			None => Ok(None),
		},
		None => Ok(None),
	}
}

// endregion: --- Lua Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_path;

	#[tokio::test]
	async fn test_lua_path_exists_true() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &[
			"./agent-script/agent-hello.aip",
			"agent-script/agent-hello.aip",
			"./sub-dir-a/agent-hello-2.aip",
			"sub-dir-a/agent-hello-2.aip",
			"./sub-dir-a/",
			"sub-dir-a",
			"./sub-dir-a/",
			"./sub-dir-a/../",
			"./sub-dir-a/..",
		];

		// -- Exec & Check
		for path in paths {
			let code = format!(r#"return aip.path.exists("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(res.as_bool().ok_or("Result should be a bool")?, "'{path}' should exist");
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_exists_false() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &["./no file .rs", "some/no-file.md", "./s do/", "no-dir/at/all"];

		for path in paths {
			let code = format!(r#"return aip.path.exists("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT exist"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_is_file_true() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &[
			"./agent-script/agent-hello.aip",
			"agent-script/agent-hello.aip",
			"./sub-dir-a/agent-hello-2.aip",
			"sub-dir-a/agent-hello-2.aip",
			"./sub-dir-a/../agent-script/agent-hello.aip",
		];

		for path in paths {
			let code = format!(r#"return aip.path.is_file("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should be a file"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_is_file_false() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &["./no-file", "no-file.txt", "sub-dir-a/"];

		for path in paths {
			let code = format!(r#"return aip.path.is_file("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT be a file"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_is_dir_true() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &["./sub-dir-a", "sub-dir-a", "./sub-dir-a/.."];

		for path in paths {
			let code = format!(r#"return aip.path.is_dir("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should be a directory"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_is_dir_false() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &[
			"./agent-hello.aipack",
			"agent-hello.aipack",
			"./sub-dir-a/agent-hello-2.aipack",
			"./sub-dir-a/other-path",
			"nofile.txt",
			"./s rc/",
		];

		for path in paths {
			let code = format!(r#"return aip.path.is_dir("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT be a directory"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_parent() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		// Fixtures: (path, expected_parent)
		let paths = &[
			("./agent-hello.aipack", "."),
			("./", ""),
			(".", ""),
			("./sub-dir/file.txt", "./sub-dir"),
			("./sub-dir/file", "./sub-dir"),
			("./sub-dir/", "."),
			("./sub-dir", "."),
		];

		for (path, expected) in paths {
			let code = format!(r#"return aip.path.parent("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			let result = res.as_str().ok_or("Should be a string")?;
			assert_eq!(result, *expected, "Parent mismatch for path: {path}");
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_join() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_path::init_module, "path")?;
		let data = [
			//
			("./", r#""file.txt""#, "./file.txt"),
			("./", r#"{"my-dir","file.txt"}"#, "./my-dir/file.txt"),
			("", r#"{"my-dir","file.txt"}"#, "my-dir/file.txt"),
			("", r#"{"my-dir/","//file.txt"}"#, "my-dir/file.txt"),
			("some-base/", r#""my-dir/","file.txt""#, "some-base/my-dir/file.txt"),
			("a", r#""b", "c""#, "a/bc"),
			("a/", r#""b/", "c/""#, "a/b/c/"),
			(
				"root/",
				r#"{"user", "docs"}, "projectA", {"report", "final.pdf"}"#,
				"root/user/docsprojectAreport/final.pdf",
			),
		];

		// -- Exec & Check
		for (base, args, expected) in data {
			let code = format!(r#"return aip.path.join("{base}", {args})"#);
			let res = eval_lua(&lua, &code)?;
			let res = res.as_str().ok_or("Should have returned string")?;

			assert_eq!(res, expected);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_split() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &[
			("some/path/to_file.md", "some/path", "to_file.md"),
			("folder/file.txt", "folder", "file.txt"),
			("justafile.md", "", "justafile.md"),
			("/absolute/path/file.log", "/absolute/path", "file.log"),
			("/file_at_root", "/", "file_at_root"),
			("trailing/slash/", "trailing", "slash"),
		];

		for (path, expected_parent, expected_filename) in paths {
			let code = format!(
				r#"
                    local parent, filename = aip.path.split("{path}")
                    return {{ parent, filename }}
                "#
			);
			let res = eval_lua(&lua, &code)?;
			let res_array = res.as_array().ok_or("Expected an array from Lua function")?;
			let parent = res_array
				.first()
				.and_then(|v| v.as_str())
				.ok_or("First value should be a string")?;
			let filename = res_array
				.get(1)
				.and_then(|v| v.as_str())
				.ok_or("Second value should be a string")?;
			assert_eq!(parent, *expected_parent, "Parent mismatch for path: {path}");
			assert_eq!(filename, *expected_filename, "Filename mismatch for path: {path}");
		}

		Ok(())
	}

	// TODO: Needs to enable (code assume res is lua, but it's serde json)
	// #[tokio::test]
	// async fn test_lua_path_parse() -> Result<()> {
	// 	// -- Setup & Fixtures
	// 	let lua = setup_lua(aip_path::init_module, "path")?;
	// 	let paths = &[
	// 		("some/path/to_file.md", "some/path", "to_file.md", "to_file", "md"),
	// 		("folder/file.txt", "folder", "file.txt", "file", "txt"),
	// 		("justafile.md", "", "justafile.md", "justafile", "md"),
	// 		("/absolute/path/file.log", "/absolute/path", "file.log", "file", "log"),
	// 		("/file_at_root", "/", "file_at_root", "file_at_root", ""),
	// 		("trailing/slash/", "trailing", "slash", "slash", ""),
	// 		(".", "", ".", ".", ""),
	// 		("", "", "", "", ""),
	// 	];

	// 	// -- Exec & Check
	// 	for (path, exp_dir, exp_name, exp_stem, exp_ext) in paths {
	// 		let code = format!(r#"return aip.path.parse("{path}")"#);
	// 		let res = eval_lua(&lua, &code)?;
	// 		let table = res.as_table().ok_or_else(|| format!("Expected a table for path: {path}"))?;

	// 		// check dir
	// 		let dir = table
	// 			.get::<_, String>("dir")
	// 			.ok_or_else(|| format!("Missing 'dir' for path: {path}"))?;
	// 		assert_eq!(dir, *exp_dir, "Dir mismatch for path: {path}");

	// 		// check name
	// 		let name = table
	// 			.get::<_, String>("name")
	// 			.ok_or_else(|| format!("Missing 'name' for path: {path}"))?;
	// 		assert_eq!(name, *exp_name, "Name mismatch for path: {path}");

	// 		// check stem
	// 		let stem = table
	// 			.get::<_, String>("stem")
	// 			.ok_or_else(|| format!("Missing 'stem' for path: {path}"))?;
	// 		assert_eq!(stem, *exp_stem, "Stem mismatch for path: {path}");

	// 		// check ext
	// 		let ext = table
	// 			.get::<_, String>("ext")
	// 			.ok_or_else(|| format!("Missing 'ext' for path: {path}"))?;
	// 		assert_eq!(ext, *exp_ext, "Ext mismatch for path: {path}");

	// 		// check path
	// 		let res_path = table
	// 			.get::<_, String>("path")
	// 			.ok_or_else(|| format!("Missing 'path' for path: {path}"))?;
	// 		assert_eq!(res_path, *path, "Path mismatch for path: {path}");
	// 	}

	// 	// Check nil case
	// 	let code_nil = r#"return aip.path.parse(nil)"#;
	// 	let res_nil = eval_lua(&lua, code_nil)?;
	// 	assert!(res_nil.is_nil(), "Expected nil for nil input");

	// 	Ok(())
	// }
}

// endregion: --- Tests
