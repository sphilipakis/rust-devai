//! Defines common helper functions for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.file` module exposes functions to interact with the file system.
//!
//! ### Functions
//!
//! - `aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord`
//! - `aip.file.save(rel_path: string, content: string)`
//! - `aip.file.append(rel_path: string, content: string)`
//! - `aip.file.ensure_exists(path: string, content?: string, options?: {content_when_empty: boolean}): FileMeta`
//! - `aip.file.list(include_globs: string | list, options?: {base_dir: string, absolute: boolean, with_meta: boolean}): list<FileMeta>`
//! - `aip.file.list_load(include_globs: string | list, options?: {base_dir: string, absolute: boolean}): list<FileRecord>`
//! - `aip.file.first(include_globs: string | list, options?: {base_dir: string, absolute: boolean}): FileMeta | nil`

use crate::Error;
use crate::dir_context::PathResolver;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::script::aip_modules::aip_file::support::{
	base_dir_and_globs, check_access_write, compute_base_dir, create_file_records, list_files_with_options,
};
use crate::support::{AsStrsExt, files};
use crate::types::{FileMeta, FileRecord};
use mlua::{FromLua, IntoLua, Lua, Value};
use simple_fs::{SPath, ensure_file_dir, iter_files};
use std::fs::write;
use std::io::Write;

/// ## Lua Documentation
///
/// Load a File Record object with its content.
///
/// ```lua
/// -- API Signature
/// aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord
/// ```
///
/// Loads the file specified by `rel_path` and returns a `FileRecord` object containing
/// the file's metadata and its content.
///
/// ### Arguments
///
/// - `rel_path: string` - The path to the file, relative to the `base_dir` or workspace root.
/// - `options?: table` - An optional table containing:
///   - `base_dir: string` (optional): The base directory from which `rel_path` is resolved. Defaults to the workspace root. Pack references (e.g., `ns@pack/`) can be used.
///
/// ### Returns
///
/// - `FileRecord: table` - A table representing the file record:
///   ```ts
///   {
///     path : string,             // Relative path used to load the file
///     name : string,             // File name with extension
///     stem : string,             // File name without extension
///     ext  : string,             // File extension
///     created_epoch_us?: number, // Creation timestamp (microseconds)
///     modified_epoch_us?: number,// Modification timestamp (microseconds)
///     size?: number,             // File size in bytes
///     content: string            // The text content of the file
///   }
///   ```
///
/// ### Example
///
/// ```lua
/// local readme = aip.file.load("doc/README.md")
/// print(readme.path)    -- Output: "doc/README.md"
/// print(readme.name)    -- Output: "README.md"
/// print(#readme.content) -- Output: <length of content>
///
/// local agent_file = aip.file.load("agent.aip", { base_dir = "ns@pack/" })
/// print(agent_file.path) -- Output: "agent.aip" (relative to the resolved base_dir)
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The `base_dir` cannot be resolved (e.g., invalid pack reference).
/// - The final file path cannot be resolved.
/// - The file does not exist or cannot be read.
/// - Metadata cannot be retrieved.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_load(
	lua: &Lua,
	runtime: &Runtime,
	rel_path: String,
	options: Option<Value>,
) -> mlua::Result<mlua::Value> {
	let base_path = compute_base_dir(runtime, options.as_ref())?;
	let full_path = match base_path {
		Some(base_path) => base_path.join(&rel_path),
		None => {
			let dir_context = runtime.dir_context();
			dir_context.resolve_path(runtime.session(), (&rel_path).into(), PathResolver::WksDir)?
		}
	};

	let rel_path = SPath::new(rel_path);

	let file_record = FileRecord::load_from_full_path(&full_path, &rel_path)?;
	let res = file_record.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Save string content to a file at the specified path.
///
/// ```lua
/// -- API Signature
/// aip.file.save(rel_path: string, content: string)
/// ```
///
/// Writes the provided `content` string to the file specified by `rel_path`.
/// The path is resolved relative to the workspace root. If the file exists, it will be overwritten.
/// If the directory structure does not exist, it will be created.
///
/// **Important Security Note:** For security reasons, this function currently restricts saving files
/// outside the workspace directory (`./`) or the shared base directory (`~/.aipack-base/`).
///
/// ### Arguments
///
/// - `rel_path: string` - The path to the file where the content should be saved, relative to the workspace root.
/// - `content: string`  - The string content to write to the file.
///
/// ### Returns
///
/// Does not return anything upon success.
///
/// ### Example
///
/// ```lua
/// -- Save documentation to a file in the 'docs' directory
/// aip.file.save("docs/new_feature.md", "# New Feature\n\nDetails about the feature.")
///
/// -- Overwrite an existing file
/// aip.file.save("config.txt", "new_setting=true")
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The path attempts to write outside the allowed workspace or base directories.
/// - The directory structure cannot be created.
/// - The file cannot be written due to permissions or other I/O errors.
/// - The operation requires a workspace context, but none is found.
///
/// ```ts
/// {
///   error: string // Error message (e.g., save file protection, permission denied, ...)
/// }
/// ```
pub(super) fn file_save(_lua: &Lua, runtime: &Runtime, rel_path: String, content: String) -> mlua::Result<()> {
	let dir_context = runtime.dir_context();
	let full_path = dir_context.resolve_path(runtime.session(), (&rel_path).into(), PathResolver::WksDir)?;

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.save requires a aipack workspace setup")?;

	check_access_write(&full_path, wks_dir)?;

	ensure_file_dir(&full_path).map_err(Error::from)?;

	write(&full_path, content).map_err(|err| Error::custom(format!("Fail to save file {rel_path}.\nCause {err}")))?;

	let rel_path = full_path.diff(wks_dir).unwrap_or(full_path);
	get_hub().publish_sync(format!("-> Lua aip.file.save called on: {}", rel_path));

	Ok(())
}

/// ## Lua Documentation
///
/// Append string content to a file at the specified path.
///
/// ```lua
/// -- API Signature
/// aip.file.append(rel_path: string, content: string)
/// ```
///
/// Appends the provided `content` string to the end of the file specified by `rel_path`.
/// The path is resolved relative to the workspace root. If the file does not exist, it will be created.
/// If the directory structure does not exist, it will be created.
///
/// ### Arguments
///
/// - `rel_path: string` - The path to the file where the content should be appended, relative to the workspace root.
/// - `content: string`  - The string content to append to the file.
///
/// ### Returns
///
/// Does not return anything upon success.
///
/// ### Example
///
/// ```lua
/// -- Append a log entry to a log file
/// aip.file.append("logs/app.log", "INFO: User logged in.\n")
///
/// -- Create a file and append if it doesn't exist
/// aip.file.append("notes.txt", "- Remember to buy milk.\n")
/// aip.file.append("notes.txt", "- Finish report.\n")
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The directory structure cannot be created.
/// - The file cannot be opened for appending (e.g., due to permissions).
/// - An I/O error occurs during writing.
///
/// ```ts
/// {
///   error: string // Error message (e.g., permission denied, I/O error)
/// }
/// ```
pub(super) fn file_append(_lua: &Lua, runtime: &Runtime, rel_path: String, content: String) -> mlua::Result<()> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&rel_path).into(), PathResolver::WksDir)?;
	ensure_file_dir(&path).map_err(Error::from)?;

	let mut file = std::fs::OpenOptions::new()
		.append(true)
		.create(true)
		.open(&path)
		.map_err(Error::from)?;

	file.write_all(content.as_bytes())?;

	// NOTE: Could be too many prints
	// get_hub().publish_sync(format!("-> Lua aip.file.append called on: {}", rel_path));

	Ok(())
}

/// ## Lua Documentation
///
/// Ensure a file exists at the given path. If it doesn't exist, create it with optional content.
/// If it exists, optionally overwrite its content if it's currently empty.
///
/// ```lua
/// -- API Signature
/// aip.file.ensure_exists(path: string, content?: string, options?: {content_when_empty?: boolean}): FileMeta
/// ```
///
/// Checks if the file at `path` (relative to the workspace root) exists.
/// - If the file does not exist:
///   - Creates the necessary directory structure.
///   - Creates the file.
///   - Writes the `content` (or an empty string if `content` is nil) to the new file.
/// - If the file exists:
///   - Checks if `options.content_when_empty` is true.
///   - If true, checks if the file is empty (contains only whitespace or is zero-length).
///   - If the file is empty and `content_when_empty` is true, overwrites the file with `content` (or an empty string if `content` is nil).
///
/// This function is intended for files, not directories.
///
/// ### Arguments
///
/// - `path: string` - The path to the file, relative to the workspace root.
/// - `content?: string` (optional) - The content to write to the file if it's created or if it's empty and `content_when_empty` is true. Defaults to an empty string if nil.
/// - `options?: table` (optional) - A table containing options:
///   - `content_when_empty?: boolean` (optional): If true, the `content` will be written to the file if the file is empty (or only contains whitespace). Defaults to `false`.
///
/// ### Returns
///
/// - `FileMeta: table` - Metadata about the file (even if it was just created).
///   ```ts
///   {
///     path : string,             // Relative path used
///     name : string,             // File name with extension
///     stem : string,             // File name without extension
///     ext  : string,             // File extension
///     created_epoch_us?: number, // Creation timestamp (microseconds)
///     modified_epoch_us?: number,// Modification timestamp (microseconds)
///     size?: number              // File size in bytes
///   }
///   ```
///
/// ### Example
///
/// ```lua
/// -- Ensure a config file exists, creating it with defaults if needed
/// local config_content = "-- Default Settings --\nenabled=true"
/// local file_meta = aip.file.ensure_exists("config/settings.lua", config_content)
/// print("Ensured file:", file_meta.path)
///
/// -- Ensure a log file exists, but don't overwrite if it has content
/// aip.file.ensure_exists("logs/activity.log")
///
/// -- Ensure a placeholder file exists, writing content only if it's empty
/// local placeholder = "-- TODO: Add content --"
/// aip.file.ensure_exists("src/module.lua", placeholder, {content_when_empty = true})
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The directory structure cannot be created.
/// - The file cannot be created or written to (e.g., due to permissions).
/// - Metadata cannot be retrieved for the file.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_ensure_exists(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	content: Option<String>,
	options: Option<EnsureExistsOptions>,
) -> mlua::Result<mlua::Value> {
	let options = options.unwrap_or_default();

	let rel_path = SPath::new(path);
	let full_path = runtime
		.dir_context()
		.resolve_path(runtime.session(), rel_path.clone(), PathResolver::WksDir)?;

	// if the file does not exist, create it.
	if !full_path.exists() {
		simple_fs::ensure_file_dir(&full_path).map_err(|err| Error::custom(err.to_string()))?;
		let content = content.unwrap_or_default();
		write(&full_path, content)?;
	}
	// if we have the options.content_when_empty flag, if empty
	else if options.content_when_empty && files::is_file_empty(&full_path)? {
		let content = content.unwrap_or_default();
		write(&full_path, content)?;
	}

	let file_meta = FileMeta::new(rel_path, &full_path);

	file_meta.into_lua(lua)
}

/// ## Lua Documentation
///
/// List file metadata (`FileMeta`) matching glob patterns.
///
/// ```lua
/// -- API Signature
/// aip.file.list(
///   include_globs: string | list<string>,
///   options?: {
///     base_dir?: string,
///     absolute?: boolean,
///     with_meta?: boolean
///   }
/// ): list<FileMeta>
/// ```
///
/// Finds files matching the `include_globs` patterns within the specified `base_dir` (or workspace root)
/// and returns a list of `FileMeta` objects containing information about each file (path, name, timestamps, size, etc.),
/// but *not* the file content.
///
/// ### Arguments
///
/// - `include_globs: string | list<string>` - A single glob pattern string or a Lua list (table) of glob pattern strings.
///   Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
/// - `options?: table` (optional) - A table containing options:
///   - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
///     Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
///   - `absolute?: boolean` (optional): If `true`, the `path` in the returned `FileMeta` objects will be absolute.
///     If `false` (default), the `path` will be relative to the `base_dir`. If a path resolves outside the `base_dir`
///     (e.g., using `../` in globs), it will be returned as an absolute path even if `absolute` is false.
///   - `with_meta?: boolean` (optional): If `false`, the function will skip fetching detailed metadata
///     (`created_epoch_us`, `modified_epoch_us`, `size`) for each file, potentially improving performance
///     if only the path information is needed. Defaults to `true`.
///
/// ### Returns
///
/// - `list<FileMeta>: table` - A Lua list (table) where each element is a `FileMeta` table:
///   ```ts
///   {
///     path : string,             // Path (relative to base_dir or absolute)
///     name : string,             // File name with extension
///     stem : string,             // File name without extension
///     ext  : string,             // File extension
///     created_epoch_us?: number, // Creation timestamp (microseconds, if with_meta=true)
///     modified_epoch_us?: number,// Modification timestamp (microseconds, if with_meta=true)
///     size?: number              // File size in bytes (if with_meta=true)
///   }
///   ```
///   The list is empty if no files match the globs.
///
/// ### Example
///
/// ```lua
/// -- List all Markdown files in the 'docs' directory (relative paths)
/// local doc_files = aip.file.list("*.md", { base_dir = "docs" })
/// for _, file in ipairs(doc_files) do
///   print(file.path) -- e.g., "guide.md", "api.md"
/// end
///
/// -- List all '.aip' files in a specific pack (absolute paths, no detailed meta)
/// local agent_files = aip.file.list("**/*.aip", {
///   base_dir = "ns@pack/",
///   absolute = true,
///   with_meta = false
/// })
/// for _, file in ipairs(agent_files) do
///   print(file.path) -- e.g., "/path/to/workspace/.aipack/ns/pack/agent1.aip"
/// end
///
/// -- List text and config files from the workspace root
/// local config_files = aip.file.list({"*.txt", "*.config"})
/// for _, file in ipairs(config_files) do
///   print(file.path, file.size) -- e.g., "notes.txt", 1024
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - `include_globs` is not a string or a list of strings.
/// - `base_dir` cannot be resolved (e.g., invalid pack reference).
/// - An error occurs during file system traversal or glob matching.
/// - Metadata cannot be retrieved (and `with_meta` is true).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_list(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;
	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	// Default is true, as we want convenient APIs, and offer user way to optimize it
	let with_meta = options.x_get_bool("with_meta").unwrap_or(true);

	let spaths = list_files_with_options(runtime, base_path.as_ref(), &include_globs.x_as_strs(), absolute)?;

	let file_metas: Vec<FileMeta> = spaths.into_iter().map(|spath| FileMeta::new(spath, with_meta)).collect();
	let res = file_metas.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// List and load files (`FileRecord`) matching glob patterns.
///
/// ```lua
/// -- API Signature
/// aip.file.list_load(
///   include_globs: string | list<string>,
///   options?: {
///     base_dir?: string,
///     absolute?: boolean
///   }
/// ): list<FileRecord>
/// ```
///
/// Finds files matching the `include_globs` patterns within the specified `base_dir` (or workspace root),
/// loads the content of each matching file, and returns a list of `FileRecord` objects.
/// Each `FileRecord` contains both metadata and the file content.
///
/// ### Arguments
///
/// - `include_globs: string | list<string>` - A single glob pattern string or a Lua list (table) of glob pattern strings.
///   Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
/// - `options?: table` (optional) - A table containing options:
///   - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
///     Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
///   - `absolute?: boolean` (optional): If `true`, the paths used internally and potentially the `path` in the returned `FileRecord`
///     objects will be absolute. If `false` (default), paths will generally be relative to the `base_dir`.
///     Note: The exact path stored in `FileRecord.path` depends on internal resolution logic, especially if paths resolve outside `base_dir`.
///
/// ### Returns
///
/// - `list<FileRecord>: table` - A Lua list (table) where each element is a `FileRecord` table:
///   ```ts
///   {
///     path : string,             // Relative or absolute path used
///     name : string,             // File name with extension
///     stem : string,             // File name without extension
///     ext  : string,             // File extension
///     created_epoch_us?: number, // Creation timestamp (microseconds)
///     modified_epoch_us?: number,// Modification timestamp (microseconds)
///     size?: number,             // File size in bytes
///     content: string            // The text content of the file
///   }
///   ```
///   The list is empty if no files match the globs.
///
/// ### Example
///
/// ```lua
/// -- Load all Markdown files in the 'docs' directory
/// local doc_files = aip.file.list_load("*.md", { base_dir = "docs" })
/// for _, file in ipairs(doc_files) do
///   print("--- File:", file.path, "---")
///   print(file.content)
/// end
///
/// -- Load all '.aip' files in a specific pack
/// local agent_files = aip.file.list_load("**/*.aip", { base_dir = "ns@pack/" })
/// for _, file in ipairs(agent_files) do
///   print("Agent Name:", file.stem)
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - `include_globs` is not a string or a list of strings.
/// - `base_dir` cannot be resolved (e.g., invalid pack reference).
/// - An error occurs during file system traversal or glob matching.
/// - Any matching file cannot be read or its metadata retrieved.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_list_load(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;

	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	let spaths = list_files_with_options(runtime, base_path.as_ref(), &include_globs.x_as_strs(), absolute)?;

	let file_records = create_file_records(runtime, spaths, base_path.as_ref(), absolute)?;

	let res = file_records.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Find the first file matching glob patterns and return its metadata (`FileMeta`).
///
/// ```lua
/// -- API Signature
/// aip.file.first(
///   include_globs: string | list<string>,
///   options?: {
///     base_dir?: string,
///     absolute?: boolean
///   }
/// ): FileMeta | nil
/// ```
///
/// Searches for files matching the `include_globs` patterns within the specified `base_dir` (or workspace root).
/// It stops searching as soon as the first matching file is found and returns its `FileMeta` object (metadata only, no content).
/// If no matching file is found, it returns `nil`.
///
/// ### Arguments
///
/// - `include_globs: string | list<string>` - A single glob pattern string or a Lua list (table) of glob pattern strings.
///   Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
/// - `options?: table` (optional) - A table containing options:
///   - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
///     Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
///   - `absolute?: boolean` (optional): If `true`, the `path` in the returned `FileMeta` object (if found) will be absolute.
///     If `false` (default), the `path` will be relative to the `base_dir`. Similar to `aip.file.list`, paths outside `base_dir` become absolute.
///
/// ### Returns
///
/// - `FileMeta: table | nil` - If a matching file is found, returns a `FileMeta` table:
///   ```ts
///   {
///     path : string,             // Path (relative to base_dir or absolute)
///     name : string,             // File name with extension
///     stem : string,             // File name without extension
///     ext  : string,             // File extension
///     created_epoch_us?: number, // Creation timestamp (microseconds)
///     modified_epoch_us?: number,// Modification timestamp (microseconds)
///     size?: number              // File size in bytes
///   }
///   ```
///   If no matching file is found, returns `nil`.
///
/// ### Example
///
/// ```lua
/// -- Find the first '.aip' file in a pack
/// local agent_meta = aip.file.first("*.aip", { base_dir = "ns@pack/" })
/// if agent_meta then
///   print("Found agent:", agent_meta.path)
///   -- To load its content:
///   -- local agent_file = aip.file.load(agent_meta.path, { base_dir = "ns@pack/" })
///   -- print(agent_file.content)
/// else
///   print("No agent file found in pack.")
/// end
///
/// -- Find any config file in the root
/// local config_meta = aip.file.first({"*.toml", "*.yaml", "*.json"}, { base_dir = "." })
/// if config_meta then
///   print("Config file:", config_meta.name)
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - `include_globs` is not a string or a list of strings.
/// - `base_dir` cannot be resolved (e.g., invalid pack reference).
/// - An error occurs during file system traversal or glob matching *before* the first file is found.
/// - Metadata cannot be retrieved for the first found file.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_first(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;

	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	let base_path = match base_path {
		Some(base_path) => base_path.clone(),
		None => runtime
			.dir_context()
			.wks_dir()
			.ok_or(Error::custom("Cannot create file records, no workspace"))?
			.clone(),
	};

	let mut sfiles = iter_files(
		&base_path,
		Some(&include_globs.iter().map(|s| s.as_str()).collect::<Vec<&str>>()),
		Some(simple_fs::ListOptions::from_relative_glob(!absolute)),
	)
	.map_err(Error::from)?;

	let Some(sfile) = sfiles.next() else {
		return Ok(Value::Nil);
	};

	let absolute_path = SPath::from(&sfile);

	let spath = if absolute {
		sfile.into()
	} else {
		sfile
			.try_diff(&base_path)
			.map_err(|err| Error::cc("Cannot diff with base_path", err))?
	};

	let res = FileMeta::new(spath, &absolute_path).into_lua(lua)?;

	Ok(res)
}

// region:    --- Options
#[derive(Debug, Default)]
pub struct EnsureExistsOptions {
	/// Set the eventual provided content if the file is empty (only whitespaces)
	pub content_when_empty: bool,
}

impl FromLua for EnsureExistsOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		let table = value
			.as_table()
			.ok_or(crate::Error::custom("EnsureExistsOptions should be a table"))?;
		let set_content_when_empty = table.get("content_when_empty")?;
		Ok(Self {
			content_when_empty: set_content_when_empty,
		})
	}
}

// endregion: --- Options

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, assert_ends_with, eval_lua, run_reflective_agent, setup_lua};
	use crate::runtime::Runtime;
	use crate::script::aip_modules::aip_file;
	use serde_json::Value;
	use simple_fs::SPath;
	use std::collections::HashMap;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_file_load_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "./agent-script/agent-hello.aip";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load("{fx_path}")"#), None).await?;

		// -- Check
		assert_contains(res.x_get_str("content")?, "from agent-hello.aip");
		assert_eq!(res.x_get_str("path")?, fx_path);
		assert_eq!(res.x_get_str("name")?, "agent-hello.aip");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_pack_ref_simple() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "ns_b@pack_b_2/main.aip";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load("{fx_path}")"#), None).await?;

		// -- Check
		assert_contains(res.x_get_str("content")?, "custom ns_b@pack_b_2 main.aip");
		assert_contains(res.x_get_str("path")?, "pack_b_2/main.aip");
		assert_eq!(res.x_get_str("name")?, "main.aip");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_pack_ref_base_support() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "ns_b@pack_b_2$base/extra/test.txt";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load("{fx_path}")"#), None).await?;

		// -- Check
		assert_contains(
			res.x_get_str("content")?,
			"Some support content - ..@..$base/extra/test.txt",
		);
		// NOTE: right now it gives back the path given (the `ns_b@pack_b_2$base/...`)
		// Will be resolve that this: ".aipack-base/support/pack/ns_b/pack_b_2/extra/test.txt",
		assert_contains(res.x_get_str("path")?, fx_path);

		assert_eq!(res.x_get_str("name")?, "test.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_pack_ref_workspace_support() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "ns_a@pack_a_1$workspace/extra/test.txt";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load("{fx_path}")"#), None).await?;

		// -- Check
		assert_contains(
			res.x_get_str("content")?,
			"Some support content - ..@..$workspace/extra/test.txt",
		);
		// NOTE: right now it gives back the path given (the `ns_a@pack_a_1$workspace/...`)
		// ".aipack/support/pack/ns_a/pack_a_1/extra/test.txt",
		assert_contains(res.x_get_str("path")?, fx_path);
		assert_eq!(res.x_get_str("name")?, "test.txt");

		Ok(())
	}

	/// Note: need the multi-thread, because save do a `get_hub().publish_sync`
	///       which does a tokio blocking (requiring multi thread)
	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join(".tmp/test_lua_file_save_simple_ok.md");
		let fx_content = "hello from test_file_save_simple_ok";

		// -- Exec
		let _res = run_reflective_agent(
			&format!(r#"return aip.file.save("{fx_dest_path}", "{fx_content}");"#),
			None,
		)
		.await?;

		// -- Check
		let file_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(file_content, fx_content);

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_ok_in_base() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.aipack_paths()
			.aipack_base_dir()
			.join(".tmp/test_lua_file_save_ok_in_base.md");
		let fx_content = "hello from test_lua_file_save_ok_in_base";

		// -- Exec
		let _res = run_reflective_agent(
			&format!(r#"return aip.file.save("{fx_dest_path}", "{fx_content}");"#),
			None,
		)
		.await?;

		// -- Check
		let file_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(file_content, fx_content);

		Ok(())
	}

	/// Note: need the multi-thread, because save do a `get_hub().publish_sync`
	///       which does a tokio blocking (requiring multi thread)
	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_err_out_workspace() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join("../.tmp/test_lua_file_save_err_out_workspace.md");
		let fx_content = "hello from test_lua_file_save_err_out_workspace";

		// -- Exec
		let res = run_reflective_agent(
			&format!(r#"return aip.file.save("{fx_dest_path}", "{fx_content}");"#),
			None,
		)
		.await;

		// -- Check
		let Err(err) = res else { panic!("Should return error") };
		assert!(err.to_string().contains("does not belong to the workspace dir"));

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_direct() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.list("{glob}");"#), None).await?;

		// -- Check
		let res_paths = to_res_paths(&res);
		assert_eq!(res_paths.len(), 2, "result length");
		assert_contains(&res_paths, "file-01.txt");
		assert_contains(&res_paths, "file-02.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_support_workspace() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "ns_b@pack_b_2$base/**/*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.list("{glob}");"#), None).await?;

		// -- Check
		let res = res.as_array().ok_or("Should return an array")?;
		assert_eq!(res.len(), 1, "result length");
		let item = res.first().ok_or("Should have one item")?;
		assert_contains(item.x_get_str("path")?, "extra/test.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_deep() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.list("{glob}");"#), None).await?;

		// -- Check
		let res_paths = to_res_paths(&res);
		assert_eq!(res_paths.len(), 3, "result length");
		assert_contains(&res_paths, "sub-dir-a/sub-sub-dir/agent-hello-3.aip");
		assert_contains(&res_paths, "sub-dir-a/sub-sub-dir/main.aip");
		assert_contains(&res_paths, "sub-dir-a/agent-hello-2.aip");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_abs_with_wild() -> Result<()> {
		// -- Fixtures
		let lua = setup_lua(aip_file::init_module, "file")?;
		let dir = SPath::new("./tests-data/config");
		let dir = dir
			.canonicalize()
			.map_err(|err| format!("Cannot canonicalize {dir:?} cause: {err}"))?;

		// This is the rust Path logic
		let glob = format!("{}/*.*", dir);
		let code = format!(r#"return aip.file.list("{glob}");"#);

		// -- Exec
		let res = eval_lua(&lua, &code)?;

		// -- Check
		let res = res.as_array().ok_or("Should be array")?;

		assert_eq!(res.len(), 1);
		let val = res.first().ok_or("Should have one item")?;
		assert_eq!(val.x_get_str("name")?, "config-current-with-aliases.toml");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_with_base_dir_all_nested() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::super::init_module, "file")?;
		let lua_code = r#"
local files = aip.file.list({"**/*.*"}, {base_dir = "sub-dir-a"})
return { files = files }
        "#;

		// -- Exec
		let res = eval_lua(&lua, lua_code)?;

		// -- Check
		let files = res
			.get("files")
			.ok_or("Should have .files")?
			.as_array()
			.ok_or("file should be array")?;

		assert_eq!(files.len(), 3, ".files.len() should be 3");

		// NOTE We cannot assume the orders as different OSes might have different orders.
		let file_by_name: HashMap<&str, &Value> =
			files.iter().map(|v| (v.x_get_str("name").unwrap_or_default(), v)).collect();

		// NOTE: Here we assume the order will be deterministic and the same across OSes (tested on Mac).
		//       This logic might need to be changed, or actually, the list might need to have a fixed order.
		let file = file_by_name.get("main.aip").ok_or("Should have 'main.aip'")?;
		assert_eq!(file.x_get_str("path")?, "sub-sub-dir/main.aip");
		let file = file_by_name.get("agent-hello-3.aip").ok_or("Should have 'agent-hello-3.aip'")?;
		assert_eq!(file.x_get_str("path")?, "sub-sub-dir/agent-hello-3.aip");
		let file = file_by_name.get("agent-hello-2.aip").ok_or("Should have 'agent-hello-2.aip'")?;
		assert_eq!(file.x_get_str("path")?, "agent-hello-2.aip");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_with_base_dir_one_level() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::super::init_module, "file")?;
		let lua_code = r#"
local files = aip.file.list({"agent-hello-*.aip"}, {base_dir = "sub-dir-a"})
return { files = files }
        "#;

		// -- Exec
		let res = eval_lua(&lua, lua_code)?;

		// -- Check
		let files = res
			.get("files")
			.ok_or("Should have .files")?
			.as_array()
			.ok_or("file should be array")?;

		assert_eq!(files.len(), 1, ".files.len() should be 1");
		// NOTE: Here we assume the order will be deterministic and the same across OSes (tested on Mac).
		//       This logic might need to be changed, or actually, the list might need to have a fixed order.
		assert_eq!(
			"agent-hello-2.aip",
			files.first().ok_or("Should have a least one file")?.x_get_str("name")?
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_first_glob_deep() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*-2.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.first("{glob}");"#), None).await?;

		// -- Check
		// let res_paths = to_res_paths(&res);
		assert_eq!(res.x_get_str("name")?, "agent-hello-2.aip");
		assert_eq!(res.x_get_str("path")?, "sub-dir-a/agent-hello-2.aip");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_first_not_found() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*-not-a-thing.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.first("{glob}")"#), None).await?;

		// -- Check
		assert_eq!(res, serde_json::Value::Null, "Should have returned null");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_file_tmp_with_ctx() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "Hello tmp content";
		let fx_path = "my-dir/some-tmp-file.aip";
		let fx_code = format!(
			r#"
local path = CTX.TMP_DIR .. "/{fx_path}"	
aip.file.save(path,"{fx_content}")
return {{
   file    = aip.file.load(path),
	 session = CTX.SESSION
}}
		"#
		);

		// -- Exec
		let res = run_reflective_agent(&fx_code, None).await?;

		// -- Check
		let content = res.x_get_str("/file/content")?;
		let path = res.x_get_str("/file/path")?;
		let session = res.x_get_str("session")?;

		assert_eq!(content, fx_content);
		assert_ends_with(path, &format!(".aipack/.session/{session}/tmp/{fx_path}"));

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_file_tmp_with_var() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "Hello tmp content";
		let fx_path = "my-dir/tmp_with_var_file.txt";
		let fx_code = format!(
			r#"
	local path = "$tmp/{fx_path}"
	aip.file.save(path,"{fx_content}")
	local files = aip.file.list_load("$tmp/**/*.*")
	return {{
	   file    = aip.file.load(path),
		 files   = files,
		 session = CTX.SESSION
	}}
			"#
		);

		// -- Exec
		let res = run_reflective_agent(&fx_code, None).await?;

		// -- Check
		let content = res.x_get_str("/file/content")?;
		let path = res.x_get_str("/file/path")?;
		let file_name = res.x_get_str("/file/name")?;
		// let session = res.x_get_str("session")?;

		assert_eq!(content, fx_content);
		assert_eq!(file_name, "tmp_with_var_file.txt");
		assert_ends_with(path, &format!("$tmp/{fx_path}"));
		// assert_ends_with(path, &format!("$tmp/{fx_path}"));

		Ok(())
	}

	// region:    --- Support for Tests

	fn to_res_paths(res: &serde_json::Value) -> Vec<&str> {
		res.as_array()
			.ok_or("should have array of path")
			.unwrap()
			.iter()
			.map(|v| v.x_get_as::<&str>("path").unwrap_or_default())
			.collect::<Vec<&str>>()
	}

	// endregion: --- Support for Tests
}

// endregion: --- Tests
