use crate::{Error, Result};
use simple_fs::SPath;

bitflags::bitflags! {
	#[derive(Clone)]
	pub struct DeleteCheck: u8 {
		/// Check if the
		const CONTAINS_AIPACK_BASE  = 0b00000001;
		const IN_CURRENT_DIR      = 0b00000010;
	}
}

/// Will do a safer delete, moving to trash if possible
/// returns true if it was deleted (if not exists, return false)
/// error if not a file
/// NOTE: On Mac, this will prompt the user to accept Finder access (which might be confusing)
pub fn safer_trash_file(path: &SPath) -> Result<bool> {
	if !path.exists() {
		return Ok(false);
	}
	if !path.is_file() {
		return Err(format!("Path '{path}' is not a file. Cannot delete with safer_delete_file.").into());
	}

	trash::delete(path).map_err(|err| format!("Cannot delete file '{path}'. Cause: {err}"))?;

	Ok(true)
}

/// Will do a safer delete, moving to trash if possible
/// returns true if it was deleted (if not exists, return false)
/// error if not a file
/// NOTE: On Mac, this will prompt the user to accept Finder access (which might be confusing)
pub fn safer_trash_dir(path: &SPath, delete_check: Option<DeleteCheck>) -> Result<bool> {
	if !path.exists() {
		return Ok(false);
	}
	if !path.is_dir() {
		return Err(format!("Path '{path}' is not a directory. Cannot delete with safer_delete_dir.").into());
	}

	// -- Check if in `.aipack-base`
	if delete_check.is_some_and(|check| check.contains(DeleteCheck::CONTAINS_AIPACK_BASE)) {
		// -- Check if int .aipack-base
		let Ok(abs_path) = path.canonicalize() else {
			return Ok(false); // for now silent
		};

		if !abs_path.as_str().contains(".aipack-base") {
			return Err(Error::custom(format!(
				"Cannot delete path '{abs_path}'. Not in .aipack-base/ path"
			)));
		}
	}

	// -- Execute the AIPack
	trash::delete(path).map_err(|err| format!("Cannot delete dir '{path}'. Cause: {err}"))?;

	Ok(true)
}
