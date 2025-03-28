use super::AipackPaths;
use crate::support::files::current_dir;
use crate::{Error, Result};
use simple_fs::SPath;

#[allow(clippy::enum_variant_names)] // to remove
pub enum PathResolver {
	CurrentDir,
	WksDir,
	#[allow(unused)]
	AipackDir,
}

#[derive(Debug, Clone)]
pub struct DirContext {
	/// Absolute path of the current_dir (pwd)
	/// (except for test, which can be mocked to another dir)
	current_dir: SPath,

	/// This is workspace `.aipack/`
	aipack_paths: AipackPaths,
}

/// Constructor/Loader
impl DirContext {
	pub fn new(aipack_paths: AipackPaths) -> Result<Self> {
		let current_dir = current_dir()?;
		Self::from_aipack_dir_and_current_dir(aipack_paths, current_dir)
	}

	/// Private to create a new DirContext
	/// Note: Only the test function will provide a mock current_dir
	fn from_aipack_dir_and_current_dir(aipack_paths: AipackPaths, current_dir: SPath) -> Result<Self> {
		let current_dir = current_dir.canonicalize()?;
		Ok(Self {
			current_dir,
			aipack_paths,
		})
	}

	#[cfg(test)]
	pub fn from_current_and_aipack_paths(current_dir: SPath, aipack_paths: AipackPaths) -> Result<Self> {
		Ok(Self {
			current_dir,
			aipack_paths,
		})
	}
}

/// Property Getters
impl DirContext {
	pub fn current_dir(&self) -> &SPath {
		&self.current_dir
	}

	/// Will always be `"./.aipack/"`
	pub fn aipack_paths(&self) -> &AipackPaths {
		&self.aipack_paths
	}

	pub fn wks_dir(&self) -> &SPath {
		self.aipack_paths().wks_dir()
	}
}

/// Resolvers
impl DirContext {
	pub fn resolve_path(&self, path: SPath, mode: PathResolver) -> Result<SPath> {
		let base_path = if path.path().is_absolute() {
			None // Path is already absolute, no base needed
		} else {
			match mode {
				PathResolver::CurrentDir => Some(self.current_dir()),
				PathResolver::WksDir => Some(self.wks_dir()),
				PathResolver::AipackDir => {
					// Get the optional AipackWksDir reference
					match self.aipack_paths().aipack_wks_dir() {
						Some(dir) => Some(dir.as_ref()), // Use AsRef<SPath>
						None => {
							return Err(Error::custom(format!(
								"Cannot resolve path relative to '.aipack' directory because it was not found in workspace '{}'",
								self.wks_dir().as_str()
							)));
						}
					}
				}
			}
		};

		let final_path = match base_path {
			Some(base) => base.join(path),
			None => path, // Path was already absolute
		};

		let path = final_path.into_normalized();

		Ok(path)
	}
}
