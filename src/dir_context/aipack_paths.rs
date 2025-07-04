use super::path_consts::PACK_INSTALLED;
use super::path_consts::{CONFIG_FILE_NAME, PACK_CUSTOM};
use crate::dir_context::path_consts::{AIPACK_DIR_NAME, PACK_DOWNLOAD};
use crate::dir_context::{AipackBaseDir, AipackWksDir};
use crate::runtime::Session;
use crate::support::files::current_dir;
use crate::{Error, Result};
use simple_fs::SPath;
use std::path::Path;

/// AipackPaths is the component that manages all of the Aipack Paths from
/// - workspace paths `./.aipack` (optional)
/// - base paths `~/.aipack-base`
#[derive(Debug, Clone)]
pub struct AipackPaths {
	/// The path to the parent workspace_dir. Can be relative, to working dir for example.
	/// NOTE: Even if `.aipack` does not exist, `wks_dir` represents the potential workspace root.
	wks_dir: Option<SPath>,

	/// This is absolute path of the `.aipack/` if it exists.
	aipack_wks_dir: Option<AipackWksDir>,

	/// This is absolute path of `~/.aipack-base/`
	aipack_base_dir: AipackBaseDir,
}

impl AipackPaths {
	/// Returns the optional reference to the workspace `.aipack` directory wrapper.
	/// Renamed from `get_aipack_wks_dir` for consistency.
	pub fn aipack_wks_dir(&self) -> Option<&AipackWksDir> {
		self.aipack_wks_dir.as_ref()
	}

	pub fn tmp_dir(&self, session: &Session) -> Option<SPath> {
		self.aipack_wks_dir()
			.map(|aip_dir| aip_dir.join(format!(".session/{}/tmp", session.as_str())))
	}
}

/// Constructor
impl AipackPaths {
	/// Will try to find the wks_dir, if not, will be none.
	/// Will create the path for the `~/.aipack-base` but won't fail if not exists
	pub fn new() -> Result<Self> {
		let wks_dir = find_wks_dir(current_dir()?)?;
		match wks_dir {
			Some(wks_dir) => Self::from_wks_dir(wks_dir),
			None => {
				let aipack_base_dir = AipackBaseDir::new()?;
				Ok(Self {
					wks_dir: None,
					aipack_wks_dir: None,
					aipack_base_dir,
				})
			}
		}
	}

	/// Creates AipackPaths from a potential workspace directory path.
	/// It determines if a `.aipack` directory exists within the given path.
	pub fn from_wks_dir(wks_path: impl AsRef<Path>) -> Result<Self> {
		// -- Compute the wks_dir
		let wks_path = wks_path.as_ref();
		// Note: We keep the check for wks_path existence, as it's the basis.
		if !wks_path.exists() {
			return Err(Error::custom(format!(
				"Cannot initialize AipackPaths, potential workspace path does not exist {}",
				wks_path.to_string_lossy()
			)));
		}
		let wks_path = wks_path.canonicalize().map_err(|err| {
			Error::custom(format!(
				"Cannot canonicalize potential wks path for {}: {}",
				wks_path.to_string_lossy(),
				err
			))
		})?;
		let wks_dir = SPath::from_std_path(wks_path)?;

		// -- Create the aipack_wks_dir
		// NOTE: this will create it even if it does not exist
		let aipack_wks_dir = Some(AipackWksDir::new_from_wks_dir(&wks_dir)?);

		// -- Compute the aipack_base_dir
		let aipack_base_dir = AipackBaseDir::new()?;

		Ok(Self {
			wks_dir: Some(wks_dir),
			aipack_wks_dir,
			aipack_base_dir,
		})
	}
}

#[cfg(test)]
impl AipackPaths {
	/// For test use the: DirContext::new_test_runtime_sandbox_01() which will use this to create the mock aipack_paths
	/// Updated to accept Option<AipackWksDir>.
	pub fn from_aipack_base_and_wks_dirs(
		base_aipack_dir: AipackBaseDir,
		wks_dir: SPath, // The workspace dir (before the .aipack/)
	) -> Result<Self> {
		let aipack_wks_dir = AipackWksDir::new_from_wks_dir(&wks_dir)?;

		// AipackWksDir is now passed directly as an option
		Ok(AipackPaths {
			wks_dir: Some(wks_dir),
			aipack_wks_dir: Some(aipack_wks_dir),
			aipack_base_dir: base_aipack_dir,
		})
	}

	/// Helper for tests to create an AipackWksDir instance for the mock setup.
	pub fn mock_aipack_wks_dir(path: SPath) -> Result<AipackWksDir> {
		AipackWksDir::new_for_test(path)
	}
}

/// Getters
impl AipackPaths {
	pub fn wks_dir(&self) -> Option<&SPath> {
		self.wks_dir.as_ref()
	}

	// `aipack_wks_dir` getter is defined above near the struct definition.

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

#[derive(Debug)]
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

	/// Returns the paths to the base config and optionally the workspace config TOML file.
	/// The base config path is always returned first.
	/// The workspace config path is returned only if `.aipack/config.toml` exists.
	pub fn get_wks_config_toml_paths(&self) -> Result<Vec<SPath>> {
		let mut paths = Vec::with_capacity(2);

		// Base config path
		let base_config_path = self.aipack_base_dir.join(CONFIG_FILE_NAME);
		// Add base path even if it doesn't exist, config loading handles that.
		paths.push(base_config_path);

		// Workspace config path (optional)
		if let Some(aipack_wks_dir) = self.aipack_wks_dir() {
			let wks_config_path = aipack_wks_dir.get_config_toml_path()?;
			// Only add workspace path if the file actually exists.
			if wks_config_path.exists() {
				paths.push(wks_config_path);
			}
		}

		Ok(paths)
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

	/// Returns the list of pack repo dirs, in the order of precedence.
	///
	/// The array will contain (if they exist):
	/// - `/path/to/wks/.aipack/pack/custom` (if `.aipack` and the `custom` dir exist)
	/// - `/path/user/home/.aipack-base/pack/custom`
	/// - `/path/user/home/.aipack-base/pack/installed`
	pub fn get_pack_repo_dirs(&self) -> Result<Vec<PackRepo>> {
		let mut dirs = Vec::new();

		// 1. Workspace custom directory: .aipack/pack/custom (optional)
		if let Some(aipack_wks_dir) = self.aipack_wks_dir() {
			let wks_custom = aipack_wks_dir.get_pack_custom_dir()?;
			if wks_custom.exists() {
				dirs.push(PackRepo::new(RepoKind::WksCustom, wks_custom));
			}
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
/// Finds the nearest parent directory containing a `.aipack` directory.
pub fn find_wks_dir(from_dir: SPath) -> Result<Option<SPath>> {
	let mut current_dir: Option<SPath> = Some(from_dir);

	while let Some(parent_dir) = current_dir {
		if parent_dir.join(AIPACK_DIR_NAME).is_dir() {
			// Try to create AipackPaths based on this parent directory.
			// This will succeed even if .aipack doesn't exist, but aipack_wks_dir will be None.
			let aipack_paths = AipackPaths::from_wks_dir(&parent_dir)?;
			// Check if the .aipack directory was found for this path.
			if aipack_paths.aipack_wks_dir().is_some() {
				return Ok(Some(parent_dir));
			}
		}

		// If AipackPaths::from_wks_dir failed (e.g., path doesn't exist, canonicalization issue),
		// or if .aipack wasn't found, proceed to the parent.

		current_dir = parent_dir.parent();
	}

	Ok(None)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::assert_ends_with;
	use crate::runtime::Runtime;

	#[tokio::test]
	async fn test_aipack_paths_from_wks_dir_exists() -> Result<()> {
		// -- Exec
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let aipack_paths = runtime.dir_context().aipack_paths();

		// -- Check wks_dir and aipack_wks_dir
		assert!(
			aipack_paths
				.wks_dir()
				.ok_or("Should have wks_dir")?
				.as_str()
				.contains("sandbox-01")
		);
		assert!(aipack_paths.aipack_wks_dir().is_some());
		assert_ends_with(aipack_paths.aipack_wks_dir().unwrap().as_str(), "sandbox-01/.aipack");

		// -- Check paths from get_wks_config_toml_paths
		// Note: test setup needs `sandbox-01/.aipack/config.toml` to exist
		let config_paths = aipack_paths.get_wks_config_toml_paths()?;
		assert_eq!(config_paths.len(), 2); // Base + Wks (assuming wks config exists)
		assert_ends_with(config_paths[0].as_str(), ".aipack-base/config.toml");
		assert_ends_with(config_paths[1].as_str(), "sandbox-01/.aipack/config.toml");

		// -- Check paths from get_pack_repo_dirs (which uses the moved methods internally)
		let pack_dirs = aipack_paths.get_pack_repo_dirs()?;
		// Assuming all test dirs exist
		assert_eq!(pack_dirs.len(), 3);
		assert_ends_with(pack_dirs[0].path().as_str(), ".aipack/pack/custom");
		assert_ends_with(pack_dirs[1].path().as_str(), ".aipack-base/pack/custom");
		assert_ends_with(pack_dirs[2].path().as_str(), ".aipack-base/pack/installed");

		Ok(())
	}

	#[tokio::test]
	async fn test_get_pack_dirs_runtime() -> Result<()> {
		// -- Setup & Fixtures
		// Runtime::new_test_runtime_sandbox_01() correctly sets up AipackPaths
		// with an existing .aipack dir.
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let aipack_paths = runtime.dir_context().aipack_paths();

		// -- Check that aipack_wks_dir is Some
		assert!(aipack_paths.aipack_wks_dir().is_some());

		// -- Exec
		let dirs = aipack_paths.get_pack_repo_dirs()?;

		// -- Check
		assert_eq!(dirs.len(), 3); // WksCustom, BaseCustom, BaseInstalled
		assert_eq!(dirs[0].kind as u8, RepoKind::WksCustom as u8);
		assert_ends_with(dirs[0].to_str(), ".aipack/pack/custom");
		assert_eq!(dirs[1].kind as u8, RepoKind::BaseCustom as u8);
		assert_ends_with(dirs[1].to_str(), ".aipack-base/pack/custom");
		assert_eq!(dirs[2].kind as u8, RepoKind::BaseInstalled as u8);
		assert_ends_with(dirs[2].to_str(), ".aipack-base/pack/installed");

		Ok(())
	}
}

// endregion: --- Tests
