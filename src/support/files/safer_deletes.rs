use crate::Result;
use simple_fs::{SPath, SaferTrashOptions};

bitflags::bitflags! {
	#[derive(Clone, Copy)]
	pub struct DeleteCheck: u8 {
		/// Check if the
		const CONTAINS_AIPACK_BASE  = 0b00000001;
		const CONTAINS_AIPACK       = 0b00000010;
	}
}

/// Will do a safer delete, moving to trash if possible
/// returns true if it was deleted (if not exists, return false)
/// error if not a file
/// NOTE: On Mac, this will prompt the user to accept Finder access (which might be confusing)
pub fn safer_trash_file(path: &SPath, delete_check: Option<DeleteCheck>) -> Result<bool> {
	let options = to_options(delete_check);
	Ok(simple_fs::safer_trash_file(path, options)?)
}

/// Will do a safer delete, moving to trash if possible
/// returns true if it was deleted (if not exists, return false)
/// error if not a file
/// NOTE: On Mac, this will prompt the user to accept Finder access (which might be confusing)
pub fn safer_trash_dir(path: &SPath, delete_check: Option<DeleteCheck>) -> Result<bool> {
	let options = to_options(delete_check);
	Ok(simple_fs::safer_trash_dir(path, options)?)
}

// region:    --- Support

fn to_options(check: Option<DeleteCheck>) -> SaferTrashOptions<'static> {
	let options = SaferTrashOptions::default();

	let Some(check) = check else {
		return options;
	};

	match (
		check.contains(DeleteCheck::CONTAINS_AIPACK_BASE),
		check.contains(DeleteCheck::CONTAINS_AIPACK),
	) {
		(true, true) => options.with_must_contain_all(&[".aipack-base", ".aipack/"]),
		(true, false) => options.with_must_contain_all(&[".aipack-base"]),
		(false, true) => options.with_must_contain_all(&[".aipack/"]),
		(false, false) => options,
	}
}

// endregion: --- Support
