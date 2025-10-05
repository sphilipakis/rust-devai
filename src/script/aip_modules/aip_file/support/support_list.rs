use crate::Error;
use crate::Result;
use crate::runtime::Runtime;
use crate::types::FileRef;
use simple_fs::{ListOptions, SPath, list_files};

/// Lists files based on provided glob patterns and options
///
/// Returns a list of files that match the globs, with paths relative to the base_dir
/// or absolute depending on the options
pub fn list_files_with_options(
	runtime: &Runtime,
	base_path: Option<&SPath>,
	include_globs: &[&str],
	absolute: bool,
	glob_sort: bool,
) -> Result<Vec<FileRef>> {
	let base_path = match base_path {
		Some(base_path) => base_path.clone(),
		None => runtime
			.dir_context()
			.wks_dir()
			.ok_or("Cannot create file records, no workspace")?
			.clone(),
	};

	let sfiles = list_files(
		&base_path,
		Some(include_globs),
		Some(ListOptions::from_relative_glob(!absolute)),
	)
	.map_err(Error::from)?;

	// Now, we put back the paths found relative to base_path
	let file_refs = sfiles
		.into_iter()
		.map(|f| {
			let smeta = f.meta().ok();
			let spath = if absolute {
				SPath::from(f)
			} else {
				//
				let diff = f.try_diff(&base_path)?;
				// if the diff goes back from base_path, then, we put the absolute path
				if diff.as_str().starts_with("..") {
					SPath::from(f)
				} else {
					diff
				}
			};

			Ok(FileRef { spath, smeta })
		})
		.collect::<simple_fs::Result<Vec<FileRef>>>()
		.map_err(|err| crate::Error::cc("Cannot list files to base", err))?;

	// sort by the globs (mke sure we use this files paths not the one before)
	let file_refs = if glob_sort {
		simple_fs::sort_by_globs(file_refs, include_globs, true)?
	} else {
		file_refs
	};

	Ok(file_refs)
}
