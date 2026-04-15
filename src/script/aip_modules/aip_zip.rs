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
//! - `aip.zip.extract(src_zip: string, dest_dir?: string): FileInfo[]`
//!   Extracts a ZIP archive into a directory and returns the extracted files.
//! - `aip.zip.read_text(src_zip: string, content_path: string): string | nil`
//!   Reads a UTF-8 text entry from a ZIP archive, returning `nil` when the entry is missing.
//! - `aip.zip.list(src_zip: string): string[]`
//!   Lists ZIP archive entry paths exactly as stored in archive order.

use crate::runtime::Runtime;
use crate::script::aip_modules::support::{check_access_write, process_path_reference};
use crate::support::zip;
use crate::types::{FileInfo, ZipOptions};
use crate::{Error, Result};
use mlua::{IntoLua, Lua, Table, Value};
use simple_fs::SPath;

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let rt = runtime.clone();
	let create_fn = lua.create_function(move |lua, (src_dir, dest_zip, options): (String, Option<String>, Option<Value>)| {
		zip_create(lua, &rt, src_dir, dest_zip, options)
	})?;
	let rt = runtime.clone();
	let extract_fn = lua.create_function(move |lua, (src_zip, dest_dir, options): (String, Option<String>, Option<Value>)| {
		zip_extract(lua, &rt, src_zip, dest_dir, options)
	})?;
	let rt = runtime.clone();
	let read_text_fn = lua
		.create_function(move |lua, (src_zip, content_path): (String, String)| zip_read_text(lua, &rt, src_zip, content_path))?;
	let rt = runtime.clone();
	let list_fn = lua.create_function(move |lua, src_zip: String| zip_list(lua, &rt, src_zip))?;

	table.set("create", create_fn)?;
	table.set("extract", extract_fn)?;
	table.set("read_text", read_text_fn)?;
	table.set("list", list_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Creates a ZIP archive from a directory.
///
/// ```lua
/// -- API Signature
/// aip.zip.create(src_dir: string, dest_zip?: string, options?: ZipOptions): FileInfo
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
/// - `options?: ZipOptions` (optional) - ZIP creation options.
///   - `globs?: string[]` - Include only files whose relative archive-style paths match at least one glob.
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
///
/// local zip_file = aip.zip.create("docs/site", "build/site.zip", {
///   globs = { "**/*.html", "assets/**/*.css" }
/// })
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The source directory does not exist or is not a directory.
/// - The destination path is outside the allowed workspace or base directories.
/// - The destination ZIP file cannot be created.
fn zip_create(
	lua: &Lua,
	runtime: &Runtime,
	src_dir: String,
	dest_zip: Option<String>,
	options: Option<Value>,
) -> mlua::Result<mlua::Value> {
	let dir_context = runtime.dir_context();
	let options = ZipOptions::from_lua(options.unwrap_or(Value::Nil), lua)
		.map_err(|e| Error::custom(format!("Failed to parse zip options.\nCause: {e}")))?;

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

	zip::zip_dir_with_globs(&src_dir_path, &dest_zip_path, options.globs.as_ref())
		.map_err(|err| Error::custom(format!("aip.zip.create failed. {err}")))?;

	let file_info = FileInfo::new(runtime.dir_context(), dest_zip_path.clone(), true);
	file_info.into_lua(lua)
}

/// ## Lua Documentation
///
/// Extracts a ZIP archive into a directory.
///
/// ```lua
/// -- API Signature
/// aip.zip.extract(src_zip: string, dest_dir?: string, options?: ZipOptions): list<FileInfo>
/// ```
///
/// Extracts the ZIP archive at `src_zip` into `dest_dir`.
///
/// If `dest_dir` is not provided, the destination defaults to a folder
/// in the same location as the source ZIP, named after the ZIP's stem (filename without extension).
///
/// For example, if `src_zip` is `"build/site.zip"`, the default destination would be `"build/site/"`.
///
/// The returned list includes extracted file entries only, in archive order.
/// Directory-only archive entries are not included.
///
/// ### Arguments
///
/// - `src_zip: string` - The source ZIP file path.
/// - `dest_dir?: string` (optional) - The destination directory for extracted content.
///   If not provided, defaults to a folder named after the ZIP stem in the same directory.
/// - `options?: ZipOptions` (optional) - ZIP extraction options.
///   - `globs?: string[]` - Extract and return only archive entries whose stored relative paths match at least one glob.
///
/// ### Returns
///
/// - `list<FileInfo>` - A list of [`FileInfo`] objects for each extracted file.
///
/// ### Example
///
/// ```lua
/// local files = aip.zip.extract("build/site.zip")
/// for _, file in ipairs(files) do
///   print(file.path) -- e.g., "build/site/index.html"
/// end
///
/// local files = aip.zip.extract("build/site.zip", "output/site")
/// for _, file in ipairs(files) do
///   print(file.name, file.size)
/// end
///
/// local html_files = aip.zip.extract("build/site.zip", "output/site", {
///   globs = { "**/*.html" }
/// })
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The source ZIP file does not exist or cannot be read.
/// - The destination path is outside the allowed workspace directory or base directories.
/// - The ZIP archive contains unsafe entry paths.
/// - Any extracted file or directory cannot be created.
fn zip_extract(
	lua: &Lua,
	runtime: &Runtime,
	src_zip: String,
	dest_dir: Option<String>,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();
	let options = ZipOptions::from_lua(options.unwrap_or(Value::Nil), lua)
		.map_err(|e| Error::custom(format!("Failed to parse zip options.\nCause: {e}")))?;

	let src_zip_path =
		process_path_reference(runtime, &src_zip).map_err(|err| Error::custom(format!("aip.zip.extract failed. {err}")))?;

	let dest_dir_path = if let Some(dest_dir) = dest_dir {
		process_path_reference(runtime, &dest_dir)
			.map_err(|err| Error::custom(format!("aip.zip.extract failed. {err}")))?
	} else {
		let parent = src_zip_path.parent().unwrap_or_else(|| SPath::new("."));
		let stem = src_zip_path.stem();
		parent.join(stem)
	};

	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.zip.extract requires a aipack workspace setup")?;
	check_access_write(&dest_dir_path, wks_dir).map_err(|err| Error::custom(format!("aip.zip.extract failed. {err}")))?;

	let extracted_files = zip::unzip_file_with_entries_and_globs(&src_zip_path, &dest_dir_path, options.globs.as_ref())
		.map_err(|err| Error::custom(format!("aip.zip.extract failed. {err}")))?;

	let file_infos: Vec<FileInfo> = extracted_files
		.into_iter()
		.map(|rel_path| {
			let full_path = dest_dir_path.join(&rel_path);
			FileInfo::new(dir_context, full_path.clone(), &full_path)
		})
		.collect();

	file_infos.into_lua(lua)
}

/// ## Lua Documentation
///
/// Reads a UTF-8 text file from inside a ZIP archive.
///
/// ```lua
/// -- API Signature
/// aip.zip.read_text(src_zip: string, content_path: string): string | nil
/// ```
///
/// Loads the archive entry at `content_path` from the ZIP file at `src_zip`.
///
/// If the requested archive entry does not exist, this function returns `nil`.
///
/// ### Arguments
///
/// - `src_zip: string` - The source ZIP file path.
/// - `content_path: string` - The path of the entry inside the ZIP archive.
///
/// ### Returns
///
/// - `string | nil` - The UTF-8 text content of the archive entry, or `nil` if the entry is not found.
///
/// ### Example
///
/// ```lua
/// local manifest = aip.zip.read_text("bundle.zip", "manifest.json")
/// if manifest ~= nil then
///   print(manifest)
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The source ZIP file does not exist or cannot be read.
/// - The archive cannot be opened.
/// - The requested archive entry exists but is not valid UTF-8.
/// - The archive entry cannot be read.
fn zip_read_text(lua: &Lua, runtime: &Runtime, src_zip: String, content_path: String) -> mlua::Result<Value> {
	let src_zip_path =
		process_path_reference(runtime, &src_zip).map_err(|err| Error::custom(format!("aip.zip.read_text failed. {err}")))?;

	match zip::extract_text_content(&src_zip_path, &content_path) {
		Ok(content) => content.into_lua(lua),
		Err(Error::ZipFileNotFound { .. }) => Ok(Value::Nil),
		Err(err) => Err(mlua::Error::external(Error::custom(format!("aip.zip.read_text failed. {err}")))),
	}
}

/// ## Lua Documentation
///
/// Lists archive entry paths from a ZIP archive.
///
/// ```lua
/// -- API Signature
/// aip.zip.list(src_zip: string): string[]
/// ```
///
/// Returns ZIP archive entry paths exactly as stored in archive order.
///
/// Directory entries are included as-is when present in the archive, for example
/// with a trailing `/`.
///
/// ### Arguments
///
/// - `src_zip: string` - The source ZIP file path.
///
/// ### Returns
///
/// - `string[]` - Archive entry paths exactly as stored in the ZIP.
///
/// ### Example
///
/// ```lua
/// local entries = aip.zip.list("bundle.zip")
/// for _, entry in ipairs(entries) do
///   print(entry)
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The source ZIP file does not exist or cannot be read.
/// - The archive cannot be opened.
/// - The archive entries cannot be enumerated.
fn zip_list(lua: &Lua, runtime: &Runtime, src_zip: String) -> mlua::Result<Value> {
	let src_zip_path =
		process_path_reference(runtime, &src_zip).map_err(|err| Error::custom(format!("aip.zip.list failed. {err}")))?;

	let entries =
		zip::list_entries(&src_zip_path).map_err(|err| Error::custom(format!("aip.zip.list failed. {err}")))?;
	entries.into_lua(lua)
}

