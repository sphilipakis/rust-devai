use crate::dir_context::aipack_paths::AipackPaths;
use crate::dir_context::resolve_pack_ref_base_path;
use crate::runtime::Session;
use crate::support::files::{current_dir, home_dir};
use crate::types::{PackRef, looks_like_pack_ref};
use crate::{Error, Result};
use simple_fs::SPath;
use std::str::FromStr;

#[allow(clippy::enum_variant_names)] // to remove
pub enum PathResolver {
	CurrentDir,
	WksDir,
	AipackDir,
}

#[derive(Debug, Clone)]
pub struct DirContext {
	/// The resolve user home dir.
	///
	/// NOTE: For now, if no home_dir found, then, it will use the root as home dir.
	home_dir: SPath,

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
			home_dir: home_dir(),
			current_dir,
			aipack_paths,
		})
	}

	#[cfg(test)]
	pub fn from_current_and_aipack_paths(current_dir: SPath, aipack_paths: AipackPaths) -> Result<Self> {
		Ok(Self {
			home_dir: home_dir(),
			current_dir,
			aipack_paths,
		})
	}
}

/// Property Getters
impl DirContext {
	pub fn home_dir(&self) -> &SPath {
		&self.home_dir
	}

	pub fn current_dir(&self) -> &SPath {
		&self.current_dir
	}

	/// Will always be `"./.aipack/"`
	pub fn aipack_paths(&self) -> &AipackPaths {
		&self.aipack_paths
	}

	pub fn wks_dir(&self) -> Option<&SPath> {
		self.aipack_paths().wks_dir()
	}

	/// Ge the wks_dir and if none, return an Error
	pub fn try_wks_dir_with_err_ctx(&self, ctx_msg: &str) -> Result<&SPath> {
		self.aipack_paths().wks_dir().ok_or_else(|| {
			format!(
				"{ctx_msg}.\nCause: No Workspace available.\nDo a 'aip init' in your project root folder to set the '.aipack/' workspace marker folder"
			)
			.into()
		})
	}
}

/// Formatters
impl DirContext {
	/// Return the display path
	/// - If .aipack/ or relative to workspace, then, relatively to workspace
	/// - If ~/.aipack-base/ then, absolute path
	pub fn get_display_path(&self, file_path: &str) -> Result<SPath> {
		let file_path = SPath::new(file_path);

		if file_path.as_str().contains(".aipack-base") {
			Ok(file_path)
		} else {
			let spath = match self.wks_dir() {
				Some(wks_dir) => file_path.try_diff(wks_dir)?,
				None => file_path,
			};
			Ok(spath)
		}
	}
}

/// Resolvers
impl DirContext {
	/// Resolve a path from this DirContext
	///
	/// This is resolve to a loadable/savable file path (if exists).
	/// Note: It wont't test if the path exists.
	///
	/// - `mode`
	///   - For pack_ref path, it will attempt to do a relative to PathResolver variant if possible,
	///   - For relative path, it will resolve relative to PathResolver variant (CurrentDir, ...)
	///   - For absolute path it will be ignored
	///
	/// - `base_dir` only get used whenthe pat is a relative path, in tis case, the mode is ignored, and this ise used
	///
	///
	pub fn resolve_path(
		&self,
		session: &Session,
		path: SPath,
		mode: PathResolver,
		base_dir: Option<&SPath>,
	) -> Result<SPath> {
		// -- First check if it starts with `~/` and resolve to home
		let path = if path.starts_with("~/") {
			path.into_replace_prefix("~", self.home_dir())
		} else {
			path
		};

		// -- Absolute Path
		let final_path = if path.is_absolute() {
			path
		}
		// -- if start with '$tmp'
		else if self.is_tmp_path(&path) {
			self.resolve_tmp_path(session, &path)?
		}
		// -- Pack ref
		else if looks_like_pack_ref(&path) {
			let pack_ref = PackRef::from_str(path.as_str())?;
			let base_path = resolve_pack_ref_base_path(self, &pack_ref)?;
			pack_ref.sub_path.map(|p| base_path.join(p)).unwrap_or(base_path)
		}
		// -- Relative path
		else {
			// Note: here is the base path of the dir takes precedence
			let base_path = if let Some(base_dir) = base_dir {
				Some(base_dir)
			} else {
				match mode {
					PathResolver::CurrentDir => Some(self.current_dir()),
					PathResolver::WksDir => {
						let wks_dir = self.try_wks_dir_with_err_ctx(&format!(
							"Cannot resolve '{path}' for workspace, because no workspace are available"
						))?;
						Some(wks_dir)
					}
					PathResolver::AipackDir => {
						// Get the optional AipackWksDir reference
						match self.aipack_paths().aipack_wks_dir() {
							Some(dir) => Some(dir.as_ref()), // Use AsRef<SPath>
							None => {
								return Err(Error::custom(format!(
									"Cannot resolve path relative to '.aipack' directory because it was not found in workspace '{}'",
									self.wks_dir()
										.map(|p| p.to_string())
										.unwrap_or_else(|| "no workspace found".to_string())
								)));
							}
						}
					}
				}
			};

			match base_path {
				Some(base) => base.join(path),
				None => path, // Path was already absolute
			}
		};

		let path = final_path.into_collapsed();

		Ok(path)
	}

	pub fn is_tmp_path(&self, path: &SPath) -> bool {
		path.starts_with("$tmp")
	}

	pub fn resolve_tmp_path(&self, session: &Session, orig_path: &SPath) -> Result<SPath> {
		let path = orig_path
			.strip_prefix("$tmp")
			.map_err(|_| Error::cc("Path not not a temp path", orig_path.to_string()))?;

		let Some(base_dir) = self.aipack_paths().tmp_dir(session) else {
			return Err(Error::custom(format!(
				"cannot resolve tmp path '{orig_path}'.\nCause: No workspace found"
			)));
		};

		Ok(base_dir.join(path))
	}

	/// Convert a potential home path to a tilde path. If not home path, return as is.
	pub fn maybe_home_path_into_tilde(&self, path: SPath) -> SPath {
		if path.is_absolute() && path.starts_with(self.home_dir()) {
			path.into_replace_prefix(self.home_dir(), "~")
		} else {
			path
		}
	}

	/// Convert a potential tilde path to a home path. If not tiled path, return as is.
	pub fn maybe_tilde_path_into_home(&self, path: SPath) -> SPath {
		if path.starts_with("~/") {
			path.into_replace_prefix("~", self.home_dir())
		} else {
			path
		}
	}
}
