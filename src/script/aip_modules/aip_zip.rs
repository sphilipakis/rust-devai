//! Defines the `zip` module, used in the Lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.zip` module exposes functions to work with ZIP archives.
//!
//! ### Functions
//!
//! - `aip.zip.create(src_dir: string, dest_zip?: string): FileInfo`
//!   Creates a ZIP archive from a directory.

use crate::runtime::Runtime;
use crate::script::aip_modules::support::{check_access_write, process_path_reference};
use crate::support::zip;
use crate::types::FileInfo;
use crate::{Error, Result};
use mlua::{IntoLua, Lua, Table};
use simple_fs::SPath;

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let rt = runtime.clone();
	let create_fn =
		lua.create_function(move |lua, (src_dir, dest_zip): (String, Option<String>)| zip_create(lua, &rt, src_dir, dest_zip))?;

	table.set("create", create_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Creates a ZIP archive from a directory.
///
/// ```lua
/// -- API Signature
/// aip.zip.create(src_dir: string, dest_zip?: string): FileInfo
/// ```
///
/// Creates a ZIP archive from the directory at `src_dir`.
///
/// If `dest_zip` is not provided, the destination defaults to a `.zip` file
/// next to the source directory, using the source directory stem.
///
/// For example, if `src_dir` is `"docs/site"`, the default destination
/// will be `"docs/site.zip"`.
///
/// ### Arguments
///
/// - `src_dir: string` - The source directory to archive.
/// - `dest_zip?: string` (optional) - The destination ZIP file path.
///   If not provided, defaults to `{src_dir_stem}.zip` next to the source directory.
///
/// ### Returns
///
/// - `FileInfo` - A [`FileInfo`] object for the created ZIP file.
///
/// ### Example
///
/// ```lua
/// local zip_file = aip.zip.create("docs/site")
/// print(zip_file.path) -- e.g., "docs/site.zip"
///
/// local zip_file = aip.zip.create("docs/site", "build/site.zip")
/// print(zip_file.name) -- e.g., "site.zip"
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The source directory does not exist or is not a directory.
/// - The destination path is outside the allowed workspace or base directories.
/// - The destination ZIP file cannot be created.
fn zip_create(lua: &Lua, runtime: &Runtime, src_dir: String, dest_zip: Option<String>) -> mlua::Result<mlua::Value> {
	let dir_context = runtime.dir_context();

	let src_dir_path =
		process_path_reference(runtime, &src_dir).map_err(|err| Error::custom(format!("aip.zip.create failed. {err}")))?;

	let dest_zip_path = if let Some(dest_zip) = dest_zip {
		process_path_reference(runtime, &dest_zip)
			.map_err(|err| Error::custom(format!("aip.zip.create failed. {err}")))?
	} else {
		let parent = src_dir_path.parent().unwrap_or_else(|| SPath::new("."));
		let stem = src_dir_path.name();
		parent.join(format!("{stem}.zip"))
	};

	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.zip.create requires a aipack workspace setup")?;
	check_access_write(&dest_zip_path, wks_dir).map_err(|err| Error::custom(format!("aip.zip.create failed. {err}")))?;

	zip::zip_dir(&src_dir_path, &dest_zip_path).map_err(|err| Error::custom(format!("aip.zip.create failed. {err}")))?;

	let file_info = FileInfo::new(runtime.dir_context(), dest_zip_path.clone(), true);
	file_info.into_lua(lua)
}

