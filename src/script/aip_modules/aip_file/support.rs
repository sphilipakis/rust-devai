use crate::Error;
use crate::Result;
use crate::dir_context::find_to_run_pack_dir;
use crate::dir_context::resolve_pack_ref_base_path;
use crate::dir_context::{DirContext, PathResolver};
use crate::pack::PackRef;
use crate::script::helpers::{get_value_prop_as_string, to_vec_of_strings};
use crate::types::{DestOptions, FileRecord};
use mlua::FromLua as _;
use mlua::{Lua, Value};
use simple_fs::{ListOptions, SPath, list_files};
use std::path::Path;
use std::str::FromStr;

/// Extracts base directory and glob patterns from options
///
/// Returns (base_path, globs)
pub fn base_dir_and_globs(
	runtime: &crate::runtime::Runtime,
	include_globs: Value,
	options: Option<&Value>,
) -> Result<(SPath, Vec<String>)> {
	let globs: Vec<String> = to_vec_of_strings(include_globs, "file::file_list globs argument")?;
	let base_dir = compute_base_dir(runtime.dir_context(), options)?;

	// Process any pack references in the globs
	let processed_globs = process_path_references(runtime.dir_context(), globs)?;

	Ok((base_dir, processed_globs))
}

/// Processes a single file path to handle pack references
///
/// - If a pack ref (with ..@../..) will process as pack ref
/// - Otherwise, just return the same path
///
/// Converts pack references like "jc@rust10x/common/file.md" to their actual paths
pub fn process_path_reference(dir_context: &DirContext, path: &str) -> Result<SPath> {
	// Check if the path starts with a potential pack reference
	if let Some(pack_ref_str) = extract_pack_reference(path) {
		// Parse the pack reference
		if let Ok(partial_pack) = PackRef::from_str(pack_ref_str) {
			// Try to find the pack directory

			// Can be the pack base dir, or the support base dir.
			if let Ok(base_dir) = resolve_pack_ref_base_path(dir_context, &partial_pack) {
				// Replace the pack reference with the actual path
				let sub_path = partial_pack.sub_path.unwrap_or_default();
				let pack_path = base_dir.join(&sub_path);

				// Get the remaining path (after the pack reference)
				let remaining_path = path.strip_prefix(pack_ref_str).unwrap_or("").trim_start_matches('/');

				// Combine the pack path with the remaining path
				let resolved_path = if remaining_path.is_empty() {
					pack_path.to_string()
				} else {
					pack_path.join(remaining_path).to_string()
				};

				return Ok(resolved_path.into());
			}
		}
	}

	// No pack reference or couldn't resolve, return the original path
	Ok(path.into())
}

/// Processes globs to handle pack references
///
/// Converts pack references like "jc@rust10x/common/**/*.md" to their actual paths
pub fn process_path_references(dir_context: &DirContext, globs: Vec<String>) -> Result<Vec<String>> {
	let mut processed_globs = Vec::with_capacity(globs.len());

	for glob in globs {
		let glob = process_path_reference(dir_context, &glob)?;
		processed_globs.push(glob.to_string());
	}

	Ok(processed_globs)
}

/// Extracts a potential pack reference from a glob string
///
/// Returns Some(reference) if the glob appears to contain a pack reference,
/// or None if it doesn't match the pattern
fn extract_pack_reference(glob: &str) -> Option<&str> {
	// Look for patterns like "namespace@package/path"
	// Stop at the first wildcard character if present

	// First check if there's an @ symbol (required for namespace@package)
	if !glob.contains('@') {
		return None;
	}

	// Find the position of the first wildcard character
	let wildcard_pos = glob.find(['*', '?', '[']);

	// Extract the substring up to the wildcard or the entire string if no wildcard
	let reference = match wildcard_pos {
		Some(pos) => {
			// Find the last path separator before the wildcard
			match glob[..pos].rfind('/') {
				Some(sep_pos) => &glob[..=sep_pos],
				None => glob, // No separator before wildcard, use the whole string
			}
		}
		None => glob,
	};

	Some(reference)
}

/// Determines the base directory to use for file operations
///
/// If options.base_dir is provided, it resolves relative to workspace
/// Otherwise it uses the workspace directory
pub fn compute_base_dir(dir_context: &DirContext, options: Option<&Value>) -> Result<SPath> {
	// the default base_path is the workspace dir.
	let workspace_path = dir_context.resolve_path("".into(), PathResolver::WksDir)?;

	// if options, try to resolve the options.base_dir
	let base_dir = get_value_prop_as_string(options, "base_dir", "aip.file... options fail")?;

	let base_dir = match base_dir {
		Some(base_dir) => {
			// Check if the base_dir is a pack reference
			if let Some(pack_ref_str) = extract_pack_reference(&base_dir) {
				if let Ok(pack_ref) = PackRef::from_str(pack_ref_str) {
					if let Ok(pack_dir) = find_to_run_pack_dir(dir_context, &pack_ref) {
						// Get the complete path by joining the pack dir with any sub path
						let sub_path = pack_ref.sub_path.unwrap_or_default();
						let remaining_path = base_dir.strip_prefix(pack_ref_str).unwrap_or("").trim_start_matches('/');

						if remaining_path.is_empty() {
							pack_dir.path.join(sub_path)
						} else {
							pack_dir.path.join(sub_path).join(remaining_path)
						}
					} else {
						// Fall back to regular path resolution if pack not found
						if crate::support::paths::is_relative(&base_dir) {
							workspace_path.join(&base_dir)
						} else {
							SPath::from(base_dir)
						}
					}
				} else {
					// Not a valid pack reference, treat as regular path
					if crate::support::paths::is_relative(&base_dir) {
						workspace_path.join(&base_dir)
					} else {
						SPath::from(base_dir)
					}
				}
			} else {
				// Not a pack reference, treat as regular path
				if crate::support::paths::is_relative(&base_dir) {
					workspace_path.join(&base_dir)
				} else {
					SPath::from(base_dir)
				}
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
				// Need to cannonicalize because we need to compute the diff
				let sfile_abs = sfile.canonicalize()?;
				let diff = sfile_abs.try_diff(base_path)?;
				// if the diff goes back from base_path, then, we put the absolute path
				// TODO: need to double check this
				let (base_path, rel_path) = if diff.as_str().starts_with("..") {
					(SPath::from(""), sfile_abs)
				} else {
					(base_path.clone(), diff)
				};
				let file_record = FileRecord::load(&base_path, &rel_path)?;
				Ok(file_record)
			}
		})
		.collect()
}

/// Resolves the destination path based on source path and destination options.
///
/// Returns a tuple of `(relative_destination_path, full_destination_path)`.
pub fn resolve_dest_path(
	lua: &Lua,
	dir_context: &DirContext,
	src_rel_path: &SPath,
	dest_value: Value,
	target_ext: &str,
	default_stem_suffix: Option<&str>,
) -> Result<(SPath, SPath)> {
	let opts: DestOptions = DestOptions::from_lua(dest_value, lua)
		.map_err(|e| Error::Custom(format!("Failed to parse destination options. Cause: {}", e)))?;

	let src_stem = Path::new(src_rel_path.as_str())
		.file_stem()
		.and_then(|s| s.to_str())
		.ok_or_else(|| Error::Custom(format!("Source path '{}' has no file stem.", src_rel_path)))?;

	let rel_dest_path: SPath = match opts {
		DestOptions::Nil => {
			let effective_stem = if let Some(def_suf) = default_stem_suffix {
				format!("{}{}", src_stem, def_suf)
			} else {
				src_stem.to_string()
			};
			let filename_part = format!("{}.{}", effective_stem, target_ext);

			if let Some(parent_dir) = src_rel_path.parent() {
				parent_dir.join(filename_part)
			} else {
				SPath::new(filename_part)
			}
		}
		DestOptions::Path(p) => p,
		DestOptions::Custom(c) => {
			let filename_part = if let Some(name_opt) = c.file_name {
				name_opt
			} else {
				let effective_stem = if let Some(opt_suf) = c.suffix {
					format!("{}{}", src_stem, opt_suf)
				} else if let Some(def_suf) = default_stem_suffix {
					if c.base_dir.is_some() {
						// If base_dir is specified, default_stem_suffix is ignored as per slim requirement
						src_stem.to_string()
					} else {
						// No base_dir, apply default_stem_suffix
						format!("{}{}", src_stem, def_suf)
					}
				} else {
					src_stem.to_string()
				};
				format!("{}.{}", effective_stem, target_ext)
			};

			if let Some(base_dir_spath) = c.base_dir {
				if base_dir_spath.as_str().is_empty() {
					SPath::new(filename_part)
				} else {
					base_dir_spath.join(filename_part)
				}
			} else if let Some(parent_dir) = src_rel_path.parent() {
				parent_dir.join(filename_part)
			} else {
				SPath::new(filename_part)
			}
		}
	};

	let full_dest_path = dir_context.resolve_path(rel_dest_path.clone(), PathResolver::WksDir)?;
	Ok((rel_dest_path, full_dest_path))
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::assert_contains;
	use crate::runtime::Runtime;
	use crate::script::aip_modules::aip_file::support::{process_path_reference, process_path_references};
	use crate::support::AsStrsExt;

	#[tokio::test]
	async fn test_lua_file_support_process_pack_references() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let dir_context = runtime.dir_context();
		let fx_globs: Vec<String> = ["ns_b@pack_b_2/main.aip", "no_ns@pack_b_2/main.aip", "**/*.txt"]
			.into_iter()
			.map(|v| v.to_string())
			.collect();

		// -- Exec
		let res = process_path_references(dir_context, fx_globs)?;

		// -- Check
		// NOTE: Now the process_path_references do not process for existences
		let res = res.x_as_strs();
		assert_eq!(res.len(), 3, "Should have three globs");
		let first = res.first().ok_or("Should have first")?;
		assert_contains(*first, "ns_b/pack_b_2/main.aip");
		assert_contains(&res, "**/*.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_support_process_path_reference() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let dir_context = runtime.dir_context();
		let fx_path = "ns_b@pack_b_2/main.aip";

		// -- Exec
		let res = process_path_reference(dir_context, fx_path)?;

		// -- Check
		assert_contains(res.as_str(), "ns_b/pack_b_2/main.aip");

		Ok(())
	}
}

// endregion: --- Tests
