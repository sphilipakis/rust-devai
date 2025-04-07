#![allow(unused)] // for now, some might be unused

// region:    --- Os Specific File

use crate::support::files::home_dir;
use simple_fs::SPath;

/// Return the os env file
/// for mac `Some("~/.zshenv")` (but absolute path)
pub fn get_os_env_file_path() -> Option<SPath> {
	let home_dir = home_dir().ok()?;

	match current_os() {
		OsType::Mac => Some(home_dir.join(".zshenv")),
		OsType::Linux => Some(home_dir.join(".bashrc")),
		OsType::Windows => None,
		OsType::Unknown => None,
	}
}

// endregion: --- Os Specific File

// region:    --- General Os Type

pub enum OsType {
	Mac,
	Linux,
	Windows,
	Unknown,
}

pub fn current_os() -> OsType {
	if cfg!(target_os = "macos") {
		OsType::Mac
	} else if cfg!(target_os = "linux") {
		OsType::Linux
	} else if cfg!(target_os = "windows") {
		OsType::Windows
	} else {
		OsType::Unknown
	}
}

pub fn is_mac() -> bool {
	cfg!(target_os = "macos")
}

pub fn is_linux() -> bool {
	cfg!(target_os = "linux")
}

pub fn is_windows() -> bool {
	cfg!(target_os = "windows")
}

// endregion: --- General Os Type
