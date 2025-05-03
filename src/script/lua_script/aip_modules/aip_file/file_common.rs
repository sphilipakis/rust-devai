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
//! - `aip.file.list(include_globs: string | list, options?: {base_dir: string, absolute: boolean}): list<FileMeta>`
//! - `aip.file.list_load(include_globs: string | list, options?: {base_dir: string, absolute: boolean}): list<FileRecord>`
//! - `aip.file.first(include_globs: string | list, options?: {base_dir: string, absolute: boolean}): FileMeta | nil`

use crate::Error;
use crate::dir_context::PathResolver;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::script::lua_script::aip_file::support::{
	base_dir_and_globs, compute_base_dir, create_file_records, list_files_with_options, process_path_reference,
};
use crate::support::{AsStrsExt, files};
use crate::types::{FileMeta, FileRecord};
use mlua::{FromLua, IntoLua, Lua, Value};
use simple_fs::{SPath, ensure_file_dir, iter_files};
use std::fs::write;
use std::io::Write;

/// ## Lua Documentation
///
/// Load a File Record object with its ontent
///
/// ```lua
/// local file = aip.file.load("doc/README.md")
/// -- file.content contains the text content of the file
/// ```
///
/// ### Arguments
///
/// - `rel_path: string` - The relative path to the file.
/// - `options?: {base_dir: string}` - Optional table with `base_dir` key to specify the base directory.
///
/// ### Returns
///
/// ```ts
/// -- FileRecord
/// {
///   path    : string,  -- The path to the file
///   content : string,  -- The text content of the file
///   name    : string,  -- The name of the file
///   stem    : string,  -- The stem of the file (name without extension)
///   ext     : string,  -- The extension of the file
/// }
/// ```
///
/// ### Error
///
/// Returns an error if the file does not exist or cannot be read.
///
pub(super) fn file_load(
	lua: &Lua,
	runtime: &Runtime,
	rel_path: String,
	options: Option<Value>,
) -> mlua::Result<mlua::Value> {
	let dir_context = runtime.dir_context();
	let base_path = compute_base_dir(dir_context, options.as_ref())?;
	let rel_path = process_path_reference(dir_context, &rel_path)?;
	let rel_path = SPath::new(rel_path);

	let file_record = FileRecord::load(&base_path, &rel_path)?;
	let res = file_record.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Save a File Content into a path
///
/// ```lua
/// aip.file.save("doc/README.md", "Some very cool documentation")
/// ```
///
/// ### Arguments
///
/// - `rel_path: string` - The relative path to the file.
/// - `content: string`  - The content to write to the file.
///
/// ### Returns
///
/// Does not return anything
///
/// ### Error
///
/// Returns an error if the file cannot be written, or if trying to save outside of workspace.
///
pub(super) fn file_save(_lua: &Lua, runtime: &Runtime, rel_path: String, content: String) -> mlua::Result<()> {
	let rel_path = SPath::new(rel_path);
	let path = runtime.dir_context().resolve_path(rel_path, PathResolver::WksDir)?;
	let dir_context = runtime.dir_context();
	ensure_file_dir(&path).map_err(Error::from)?;

	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.save requires a aipack workspace setup")?;

	if let Some(rel_path) = path.diff(wks_dir) {
		if rel_path.as_str().starts_with("..") {
			// allow the .aipack-base
			if !path.as_str().contains(".aipack-base") {
				return Err(Error::custom(format!(
            "Save file protection - The path `{rel_path}` does not belong to the workspace dir `{wks_dir}` or to the .aipack-base.\nCannot save file out of workspace or aipack base at this point"
        ))
        .into());
			}
		}
	}

	write(&path, content)?;

	let rel_path = path.diff(wks_dir).unwrap_or(path);
	get_hub().publish_sync(format!("-> Lua aip.file.save called on: {}", rel_path));

	Ok(())
}

/// ## Lua Documentation
///
/// Append content to a file at a specified path
///
/// ```lua
/// aip.file.append("doc/README.md", "Appended content to the file")
/// ```
///
/// ### Arguments
///
/// - `rel_path: string` - The relative path to the file.
/// - `content: string`  - The content to append to the file.
///
/// ### Returns
///
/// Does not return anything
///
/// ### Error
///
/// Returns an error if the file cannot be opened or written to.
///
pub(super) fn file_append(_lua: &Lua, runtime: &Runtime, rel_path: String, content: String) -> mlua::Result<()> {
	let path = runtime.dir_context().resolve_path((&rel_path).into(), PathResolver::WksDir)?;
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
/// Ensure a file exists at the given path, and if not create it with an optional content
/// (only to be used for file, do not use for directory)
///
/// ```lua
/// aip.file.ensure_exists(path, optional_content, options) -- FileMeta
/// ```
///
/// ### Arguments
///
/// - `path: string` - The relative path to the file.
/// - `content?: string` - Optional content to write to the file if it does not exist.
/// - `options?: {content_when_empty: boolean}` - Optional flags to set content only if the file is empty.
///
/// ### Returns
///
/// ```ts
/// -- FileMeta
/// {
///   path : string,  -- The path to the file
///   name : string,  -- The name of the file
///   stem : string,  -- The stem of the file (name without extension)
///   ext  : string   -- The extension of the file
/// }
/// ```
///
/// ### Error
///
/// Returns an error if the file cannot be created or written to.
///
pub(super) fn file_ensure_exists(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	content: Option<String>,
	options: Option<EnsureExistsOptions>,
) -> mlua::Result<mlua::Value> {
	let options = options.unwrap_or_default();
	let rel_path = SPath::new(path);
	let full_path = runtime.dir_context().resolve_path(rel_path.clone(), PathResolver::WksDir)?;

	// if the file does not exist, create it.
	if !full_path.exists() {
		simple_fs::ensure_file_dir(&full_path).map_err(|err| Error::custom(err.to_string()))?;
		let content = content.unwrap_or_default();
		write(&full_path, content)?;
	}
	// if we have the options.content_when_empty flag, if empty
	else if options.content_when_empty && files::is_file_empty(&full_path)? {
		let content = content.unwrap_or_default();
		write(full_path, content)?;
	}

	let file_meta = FileMeta::from(rel_path);

	file_meta.into_lua(lua)
}

/// ## Lua Documentation
///
/// List a set of file reference (no content) for a given glob
///
/// ```lua
/// let all_doc_file = aip.file.list("doc/**/*.md", options: {base_dir?: string, absolute?: bool})
/// ```
///
/// ### Arguments
///
/// - `include_globs: string | list` - A glob pattern or a list of glob patterns to include files.
/// - `options?: {base_dir: string, absolute: boolean}` - Optional table with `base_dir` and `absolute` keys.
///
/// ### Returns
///
/// ```ts
/// -- An array/table of FileMeta
/// {
///   path : string,  -- The path to the file
///   name : string,  -- The name of the file
///   stem : string,  -- The stem of the file (name without extension)
///   ext  : string   -- The extension of the file
/// }
/// ```
///
/// To get the content of files, needs iterate and load each
///
/// ### Error
///
/// Returns an error if the glob pattern is invalid or the files cannot be listed.
///
pub(super) fn file_list(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;

	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	let sfiles = list_files_with_options(&base_path, &include_globs.x_as_strs(), absolute)?;

	let file_metas: Vec<FileMeta> = sfiles.into_iter().map(FileMeta::from).collect();
	let res = file_metas.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// List a set of file reference (no content) for a given glob and load them
///
/// ```lua
/// let all_doc_file = aip.file.list_load("doc/**/*.md", options: {base_dir?: string, absolute?: bool})
/// ```
///
/// ### Arguments
///
/// - `include_globs: string | list` - A glob pattern or a list of glob patterns to include files.
/// - `options?: {base_dir: string, absolute: boolean}` - Optional table with `base_dir` and `absolute` keys.
///
/// ### Returns
///
/// ```ts
/// -- An array/table of FileRecord
/// {
///   path    : string,  -- The path to the file
///   name    : string,  -- The name of the file
///   stem    : string,  -- The stem of the file (name without extension)
///   ext     : string,  -- The extension of the file
///   content : string   -- The content of the file
/// }
/// ```
///
/// ### Error
///
/// Returns an error if the glob pattern is invalid or the files cannot be listed or loaded.
///
pub(super) fn file_list_load(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;

	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	let sfiles = list_files_with_options(&base_path, &include_globs.x_as_strs(), absolute)?;

	let file_records = create_file_records(sfiles, &base_path, absolute)?;

	let res = file_records.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Return the first FileMeta or Nil
///
/// ```lua
/// let first_doc_file = aip.file.first("doc/**/*.md", options: {base_dir?: string, absolute?: bool})
/// ```
///
/// ### Arguments
///
/// - `include_globs: string | list` - A glob pattern or a list of glob patterns to include files.
/// - `options?: {base_dir: string, absolute: boolean}` - Optional table with `base_dir` and `absolute` keys.
///
/// ### Returns
///
/// ```ts
/// -- FileMeta or Nil
/// {
///   path : string,  -- The path to the file
///   name : string,  -- The name of the file
///   stem : string,  -- The stem of the file (name without extension)
///   ext  : string   -- The extension of the file
/// }
/// ```
///
/// To get the file record with .content, do
///
/// ```lua
/// let file = aip.file.load(file_meta.path)
/// ```
///
/// ### Error
///
/// Returns an error if the glob pattern is invalid or the files cannot be listed.
///
pub(super) fn file_first(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;

	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	let mut sfiles = iter_files(
		&base_path,
		Some(&include_globs.iter().map(|s| s.as_str()).collect::<Vec<&str>>()),
		Some(simple_fs::ListOptions::from_relative_glob(!absolute)),
	)
	.map_err(Error::from)?;

	let Some(sfile) = sfiles.next() else {
		return Ok(Value::Nil);
	};

	let spath = if absolute {
		sfile.into()
	} else {
		sfile
			.try_diff(&base_path)
			.map_err(|err| Error::cc("Cannot diff with base_path", err))?
	};

	let res = FileMeta::from(spath).into_lua(lua)?;

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

	use crate::_test_support::{assert_contains, eval_lua, run_reflective_agent, setup_lua};
	use crate::runtime::Runtime;
	use crate::script::lua_script::aip_file;
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
		assert_contains(
			res.x_get_str("path")?,
			".aipack-base/support/pack/ns_b/pack_b_2/extra/test.txt",
		);
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
		assert_contains(
			res.x_get_str("path")?,
			".aipack/support/pack/ns_a/pack_a_1/extra/test.txt",
		);
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
