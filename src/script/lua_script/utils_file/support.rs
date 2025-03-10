use crate::Error;
use crate::Result;
use crate::dir_context::{DirContext, PathResolver};
use crate::script::lua_script::helpers::{get_value_prop_as_string, to_vec_of_strings};
use crate::types::FileRecord;
use mlua::Value;
use simple_fs::{ListOptions, SPath, list_files};

/// Extracts base directory and glob patterns from options
///
/// Returns (base_path, globs)
pub fn base_dir_and_globs(
	ctx: &crate::run::RuntimeContext,
	include_globs: Value,
	options: Option<&Value>,
) -> Result<(SPath, Vec<String>)> {
	let globs: Vec<String> = to_vec_of_strings(include_globs, "file::file_list globs argument")?;
	let base_dir = compute_base_dir(ctx.dir_context(), options)?;
	Ok((base_dir, globs))
}

/// Determines the base directory to use for file operations
///
/// If options.base_dir is provided, it resolves relative to workspace
/// Otherwise it uses the workspace directory
pub fn compute_base_dir(dir_context: &DirContext, options: Option<&Value>) -> Result<SPath> {
	// the default base_path is the workspace dir.
	let workspace_path = dir_context.resolve_path("".into(), PathResolver::WksDir)?;

	// if options, try to resolve the options.base_dir
	let base_dir = get_value_prop_as_string(options, "base_dir", "utils.file... options fail")?;

	let base_dir = match base_dir {
		Some(base_dir) => {
			if crate::support::paths::is_relative(&base_dir) {
				workspace_path.join(&base_dir)
			} else {
				SPath::from(base_dir)
			}
		}
		None => workspace_path,
	};

	Ok(base_dir)
}

/// Lists files based on provided glob patterns and options
///
/// Returns a list of files that match the globs, with paths relative to the base_dir
/// or absolute depending on the options
pub fn list_files_with_options(base_path: &SPath, include_globs: &[&str], absolute: bool) -> Result<Vec<SPath>> {
	let sfiles = list_files(
		base_path,
		Some(include_globs),
		Some(ListOptions::from_relative_glob(!absolute)),
	)
	.map_err(Error::from)?;

	// Now, we put back the paths found relative to base_path
	let sfiles = sfiles
		.into_iter()
		.map(|f| {
			if absolute {
				Ok(SPath::from(f))
			} else {
				//
				let diff = f.try_diff(base_path)?;
				// if the diff goes back from base_path, then, we put the absolute path
				if diff.as_str().starts_with("..") {
					Ok(SPath::from(f))
				} else {
					Ok(diff)
				}
			}
		})
		.collect::<simple_fs::Result<Vec<SPath>>>()
		.map_err(|err| crate::Error::cc("Cannot list files to base", err))?;

	Ok(sfiles)
}

/// Creates a vector of FileRecords from file paths
///
/// Takes a list of file paths and base path, loads content and creates FileRecord objects
pub fn create_file_records(sfiles: Vec<SPath>, base_path: &SPath, absolute: bool) -> Result<Vec<FileRecord>> {
	sfiles
		.into_iter()
		.map(|sfile| -> Result<FileRecord> {
			if absolute {
				// Note the first path won't be taken in account by FileRecord (will need to make that better typed)
				let file_record = FileRecord::load(&SPath::from(""), &sfile)?;
				Ok(file_record)
			} else {
				//
				let diff = sfile.try_diff(base_path)?;
				// if the diff goes back from base_path, then, we put the absolute path
				// TODO: need to double check this
				let (base_path, rel_path) = if diff.as_str().starts_with("..") {
					(SPath::from(""), sfile)
				} else {
					(base_path.clone(), diff)
				};
				let file_record = FileRecord::load(&base_path, &rel_path)?;
				Ok(file_record)
			}
		})
		.collect()
}
