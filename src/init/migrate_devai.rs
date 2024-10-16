use crate::init::{DEVAI_AGENT_CUSTOM_DIR, DEVAI_AGENT_DEFAULT_DIR};
use crate::Result;
use simple_fs::{ensure_dir, list_files, SPath};
use std::fs;
use std::path::Path;

pub const DEVAI_0_1_0_AGENT_DEFAULTS_DIR: &str = ".devai/defaults";
pub const DEVAI_0_1_0_AGENT_CUSTOMS_DIR: &str = ".devai/customs";

pub const DEVAI_0_1_0_DEPRECATED_DIR: &str = ".devai/_deprecated_v0_1_0";

pub fn migrate_devai_0_1_0_if_needed() -> Result<bool> {
	// -- migrate the default command agents
	let defaults_migrated = migrate_agent_dir(DEVAI_0_1_0_AGENT_DEFAULTS_DIR, DEVAI_AGENT_DEFAULT_DIR)?;
	archive_agent_dir(DEVAI_0_1_0_AGENT_DEFAULTS_DIR)?;

	// -- migrate the custom command agents
	let customs_migrated = migrate_agent_dir(DEVAI_0_1_0_AGENT_CUSTOMS_DIR, DEVAI_AGENT_CUSTOM_DIR)?;
	archive_agent_dir(DEVAI_0_1_0_AGENT_CUSTOMS_DIR)?;

	Ok(defaults_migrated || customs_migrated)
}

/// This is a v0.1.0 to v0.1.1 migration
/// For example (from .devai/customs/.. to ./devai/custom/command-agent/..)
/// - Copy the legacy `*.md` at the root of the folder to the new target folder with `*.devai`
///    -  Only the direct decending .md files
/// - Move the whole legacy folder `.devai/customs` to the `.devai/deprecated_v0_1_0/customs`
fn migrate_agent_dir(old_dir: impl AsRef<Path>, dest_dir: impl AsRef<Path>) -> Result<bool> {
	let old_dir = old_dir.as_ref();
	if !old_dir.exists() {
		return Ok(false);
	}

	let dest_dir = dest_dir.as_ref();

	ensure_dir(dest_dir)?;

	let mut at_least_one = false;

	let legacy_files = list_files(old_dir, Some(&["*.md"]), None);

	for file in legacy_files? {
		let dest_file_name = format!("{}.devai", file.file_stem());
		let dest_file = dest_dir.join(&dest_file_name);

		// we skip
		if dest_file.exists() {
			continue;
		}

		std::fs::copy(file.path(), dest_file)?;

		if !at_least_one {
			at_least_one = true;
		}
	}

	Ok(at_least_one)
}

fn archive_agent_dir(old_dir: impl AsRef<Path>) -> Result<bool> {
	let old_dir = old_dir.as_ref();
	if !old_dir.exists() {
		return Ok(false);
	}

	let old_dir = SPath::from_path(old_dir)?;

	let dest_base_dir = Path::new(DEVAI_0_1_0_DEPRECATED_DIR);
	ensure_dir(dest_base_dir)?;

	let dest_dir = dest_base_dir.join(old_dir.file_name());

	fs::rename(old_dir, dest_dir)?;

	Ok(true)
}
