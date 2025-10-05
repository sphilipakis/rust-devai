//! FileRef is more of an internal file for for the list_file than a type exported to lua
//! FileInfo and FileRecord are the one exported to lua.

use simple_fs::{SMeta, SPath};

#[derive(Debug, Clone)]
pub struct FileRef {
	pub spath: SPath,
	pub smeta: Option<SMeta>,
}

impl FileRef {
	pub fn meta(&self) -> Option<&SMeta> {
		self.smeta.as_ref()
	}
}

// region:    --- Traits

impl AsRef<SPath> for FileRef {
	fn as_ref(&self) -> &SPath {
		&self.spath
	}
}

// endregion: --- Traits
