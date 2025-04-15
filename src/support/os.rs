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

/// Returns a platform identifier string like "macos-aarch64", "linux-x86_64", "windows-x86_64".
pub fn get_platform() -> &'static str {
	// -- Mac
	if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
		"aarch64-apple-darwin"
	} else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
		"x86_64-apple-darwin"
	}
	// -- Linux
	else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
		"x86_64-unknown-linux-gnu"
	} else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
		"aarch64-unknown-linux-gnu" // Assuming support for Linux ARM
	}
	// -- Windows
	else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
		"x86_64-pc-windows-msvc"
	} else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
		"aarch64-pc-windows-msvc"
	} else {
		"unknown-unknown" // Fallback for unsupported platforms
	}
}

#[allow(unused)]
pub fn is_unix() -> bool {
	cfg!(target_os = "macos") || cfg!(target_os = "linux")
}

#[allow(unused)]
pub fn is_mac() -> bool {
	cfg!(target_os = "macos")
}

#[allow(unused)]
pub fn is_linux() -> bool {
	cfg!(target_os = "linux")
}

#[allow(unused)]
pub fn is_windows() -> bool {
	cfg!(target_os = "windows")
}

// endregion: --- General Os Type

// region:    --- Messages

pub fn get_set_api_key_message() -> &'static str {
	match current_os() {
		OsType::Mac | OsType::Linux | OsType::Unknown => {
			r#"You can set environment variable like:

export OPENAI_API_KEY="YOUR_OPENAI_KEY_HERE"
		"#
		}
		OsType::Windows => {
			r#"You can set environment variable like (Assuming PowerShell):

$env:OPENAI_API_KEY = 'YOUR_OPENAI_KEY_HERE'
		"#
		}
	}
}

// endregion: --- Messages
