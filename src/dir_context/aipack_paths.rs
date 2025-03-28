use super::path_consts::PACK_INSTALLED;
use super::path_consts::{AIPACK_BASE, AIPACK_DIR_NAME, CONFIG_FILE_NAME, PACK_CUSTOM};
use crate::dir_context::path_consts::PACK_DOWNLOAD;
use crate::dir_context::{AipackBaseDir, AipackWksDir};
use crate::{Error, Result};
use home::home_dir;
use simple_fs::SPath;
use std::path::Path;

/// AipackPaths is the component that manages all of the Aipack Paths from
/// - workspace paths `./.aipack`
/// - base paths `~/.aipack-base`
///
/// TODO: Might want to explore if we can make the wks optional.
#[derive(Debug, Clone)]
pub struct AipackPaths {
	/// The path to the parent workspace_dir. Can be relative, to working dir for example.
	wks_dir: SPath,

	/// This is absolute path of the `.aipack/`
	aipack_wks_dir: AipackWksDir,

	/// This is absolute path of `~/.aipack-base/`
	aipack_base_dir: AipackBaseDir,
}

impl AipackPaths {
	pub fn get_aipack_wks_dir(&self) -> &AipackWksDir {
		&self.aipack_wks_dir
	}
}

/// Constructor
impl AipackPaths {
	pub fn from_wks_dir(wks_path: impl AsRef<Path>) -> Result<Self> {
		// -- Compute the wks_dir
		let wks_path = wks_path.as_ref();
		if !wks_path.exists() {
			return Err(Error::custom(format!(
				"Cannot run aip, workspace path does not exist {}",
				wks_path.to_string_lossy()
			)));
		}
		let wks_path = wks_path.canonicalize().map_err(|err| {
			Error::custom(format!(
				"Cannot canonicalize wks path for {}: {}",
				wks_path.to_string_lossy(),
				err
			))
		})?;
		let wks_dir = SPath::from_std_path(wks_path)?;

		// -- Compute the aipack_wks_dir
		let aipack_wks_dir = AipackWksDir::new(&wks_dir)?;

		// -- Compute the aipack_base_dir
		let aipack_base_dir = AipackBaseDir::new()?;

		Ok(Self {
			wks_dir,
			aipack_wks_dir,
			aipack_base_dir,
		})
	}
}

#[cfg(test)]
impl AipackPaths {
	/// For test use the: DirContext::new_test_runtime_sandbox_01() which will use this to create the mock aipack_paths
	pub fn from_aipack_base_and_wks_dirs(base_aipack_dir: AipackBaseDir, wks_aipack_dir_path: SPath) -> Result<Self> {
		let wks_dir = wks_aipack_dir_path
			.parent()
			.ok_or("Should have parent wks_dir (it's for test)")?;
		let aipack_wks_dir = AipackWksDir::new_for_test(wks_aipack_dir_path)?;
		Ok(AipackPaths {
			wks_dir,
			aipack_wks_dir,
			aipack_base_dir: base_aipack_dir,
		})
	}
}

/// Getters
impl AipackPaths {
	pub fn wks_dir(&self) -> &SPath {
		&self.wks_dir
	}

	pub fn aipack_wks_dir(&self) -> &AipackWksDir {
		&self.aipack_wks_dir
	}

	#[allow(unused)]
	pub fn aipack_base_dir(&self) -> &AipackBaseDir {
		&self.aipack_base_dir
	}
}

#[derive(Debug, Clone, Copy)]
pub enum RepoKind {
	WksCustom,
	BaseCustom,
	BaseInstalled,
}

impl RepoKind {
	pub fn to_pretty_lower(self) -> String {
		match self {
			Self::WksCustom => "workspace custom - .aipack/pack/custom",
			Self::BaseCustom => "base custom - ~/.aipack-base/pack/custom",
			Self::BaseInstalled => "base installed - ~/.aipack-base/pack/installed",
		}
		.to_string()
	}
}

pub struct PackRepo {
	pub kind: RepoKind,
	pub path: SPath,
}

/// Constructor & Getters
impl PackRepo {
	pub fn new(kind: RepoKind, path: SPath) -> Self {
		Self { kind, path }
	}

	#[allow(unused)]
	pub fn to_str(&self) -> &str {
		self.path.as_str()
	}

	pub fn path(&self) -> &SPath {
		&self.path
	}
}

/// Get compute path/s
impl AipackPaths {
	// region:    --- Workspace Files & Dirs
	// NOTE: get_wks_config_toml_path moved to AipackWksDir as get_config_toml_path
	// NOTE: get_wks_pack_custom_dir moved to AipackWksDir as get_pack_custom_dir

	/// Returns the paths to the base config and workspace config TOML files.
	pub fn get_wks_config_toml_paths(&self) -> Result<Vec<SPath>> {
		let wks_config_path = self.aipack_wks_dir.get_config_toml_path()?;
		let base_config_path = self.aipack_base_dir.join(CONFIG_FILE_NAME);
		Ok(vec![base_config_path, wks_config_path])
	}
	// endregion: --- Workspace Files & Dirs

	// region:    --- Base Files & Dirs

	pub fn get_base_pack_custom_dir(&self) -> Result<SPath> {
		let dir = self.aipack_base_dir.join(PACK_CUSTOM);
		Ok(dir)
	}

	pub fn get_base_pack_installed_dir(&self) -> Result<SPath> {
		let dir = self.aipack_base_dir.join(PACK_INSTALLED);
		Ok(dir)
	}

	pub fn get_base_pack_download_dir(&self) -> Result<SPath> {
		let dir = self.aipack_base_dir.join(PACK_DOWNLOAD);
		Ok(dir)
	}

	// endregion: --- Base Files & Dirs

	/// Returns the list of pack dirs, in the order of precedence.
	///
	/// The array will contain:
	/// - `/path/to/wks/.aipack/pack/custom`
	/// - `/path/user/home/.aipack-base/pack/custom`
	/// - `/path/user/home/.aipack-base/pack/installed`
	pub fn get_pack_repo_dirs(&self) -> Result<Vec<PackRepo>> {
		let mut dirs = Vec::new();

		// 1. Workspace custom directory: .aipack/pack/custom
		let wks_custom = self.aipack_wks_dir.get_pack_custom_dir()?;
		if wks_custom.exists() {
			dirs.push(PackRepo::new(RepoKind::WksCustom, wks_custom));
		}

		// 2. Base custom directory: ~/.aipack-base/pack/custom
		let base_custom = self.get_base_pack_custom_dir()?;
		if base_custom.exists() {
			dirs.push(PackRepo::new(RepoKind::BaseCustom, base_custom));
		}

		// 3. Base installed directory: ~/.aipack-base/pack/installed
		let base_installed = self.get_base_pack_installed_dir()?;
		if base_installed.exists() {
			dirs.push(PackRepo::new(RepoKind::BaseInstalled, base_installed));
		}

		Ok(dirs)
	}
}

/// Return an option of spath tuple as (workspace_dir, aipack_dir)
pub fn find_wks_dir(from_dir: SPath) -> Result<Option<SPath>> {
	let mut tmp_dir: Option<SPath> = Some(from_dir);

	while let Some(parent_dir) = tmp_dir {
		// Note: This constructs AipackPaths just to check existence.
		// This might involve canonicalization, which could be optimized if performance becomes an issue.
		if let Ok(aipack_paths) = AipackPaths::from_wks_dir(&parent_dir) {
			if aipack_paths.aipack_wks_dir().exists() {
				return Ok(Some(parent_dir));
			}
		}
		// If AipackPaths::from_wks_dir failed (e.g., path doesn't exist, canonicalization issue),
		// we should still proceed to the parent.

		tmp_dir = parent_dir.parent();
	}

	Ok(None)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{SANDBOX_01_WKS_DIR, assert_ends_with};
	use crate::runtime::Runtime;

	#[tokio::test]
	async fn test_aipack_dir_simple() -> Result<()> {
		// -- Exec
		let aipack_paths = AipackPaths::from_wks_dir(SANDBOX_01_WKS_DIR)?;

		// -- Check paths from get_wks_config_toml_paths
		let config_paths = aipack_paths.get_wks_config_toml_paths()?;
		assert_eq!(config_paths.len(), 2);
		// Base config
		assert_ends_with(
			config_paths[0].as_str(),
			".aipack-base/config.toml", // Adjusted assertion
		);
		// Workspace config
		assert_ends_with(config_paths[1].as_str(), "tests-data/sandbox-01/.aipack/config.toml");

		// -- Check paths from get_pack_repo_dirs (which uses the moved methods internally)
		let pack_dirs = aipack_paths.get_pack_repo_dirs()?;
		assert_eq!(pack_dirs.len(), 3);
		assert_ends_with(pack_dirs[0].path().as_str(), ".aipack/pack/custom"); // Check wks custom path
		assert_ends_with(pack_dirs[1].path().as_str(), ".aipack-base/pack/custom"); // Check base custom path
		assert_ends_with(pack_dirs[2].path().as_str(), ".aipack-base/pack/installed"); // Check base installed path

		Ok(())
	}

	#[tokio::test]
	async fn test_get_pack_dirs() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let aipack_paths = runtime.dir_context().aipack_paths();

		// -- Exec
		let dirs = aipack_paths.get_pack_repo_dirs()?;

		// -- Check
		assert_eq!(dirs.len(), 3);
		assert_ends_with(dirs[0].to_str(), ".aipack/pack/custom");
		assert_ends_with(dirs[1].to_str(), ".aipack-base/pack/custom");
		assert_ends_with(dirs[2].to_str(), ".aipack-base/pack/installed");

		Ok(())
	}

	// TODO: Add test for find_wks_dir
}

// endregion: --- Tests
