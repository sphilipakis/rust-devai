use crate::Error;
use crate::dir_context::PathResolver;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::script::aip_modules::aip_file::support::{
	base_dir_and_globs, check_access_write, compute_base_dir, create_file_records, list_files_with_options,
};
use crate::support::{AsStrsExt, files, text};
use crate::types::{FileMeta, FileRecord};
use mlua::{FromLua, IntoLua, Lua, Value};
use simple_fs::{SPath, ensure_file_dir, iter_files};
use std::fs::write;
use std::io::Write;

pub(super) fn file_save_changes(_lua: &Lua, runtime: &Runtime, rel_path: String, changes: String) -> mlua::Result<()> {
	let dir_context = runtime.dir_context();
	let full_path = dir_context.resolve_path(runtime.session(), (&rel_path).into(), PathResolver::WksDir)?;

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.save requires a aipack workspace setup")?;

	check_access_write(&full_path, wks_dir)?;

	ensure_file_dir(&full_path).map_err(Error::from)?;

	let content = if full_path.exists() {
		let content = simple_fs::read_to_string(&full_path).map_err(Error::custom)?;
		text::apply_changes(&content, changes)?
	} else {
		changes
	};

	write(&full_path, content).map_err(|err| Error::custom(format!("Fail to save file {rel_path}.\nCause {err}")))?;

	let rel_path = full_path.diff(wks_dir).unwrap_or(full_path);
	get_hub().publish_sync(format!("-> Lua aip.file.save called on: {}", rel_path));

	Ok(())
}
