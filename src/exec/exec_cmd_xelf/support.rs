use crate::Result;
use simple_fs::SPath;
use std::fs;

pub(super) fn atomic_replace(src: &SPath, dest: &SPath) -> Result<()> {
	fs::rename(src, dest).map_err(|err| {
		format!(
			"Failed to replace '{}' with '{}'. Cause: {}.\n\
					 On Windows, make sure all 'aip' processes are terminated before updating.",
			dest, src, err
		)
	})?;
	Ok(())
}
