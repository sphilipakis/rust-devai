use crate::dir_context::DirContext;
use crate::exec::packer::pack_toml::parse_validate_pack_toml;
use crate::exec::packer::{PackToml, support};
use crate::exec::packer::support::{PackUri, download_from_repo, download_pack, resolve_local_path};
use crate::support::files::{DeleteCheck, safer_trash_dir, safer_trash_file};
use crate::support::zip;
use crate::{Error, Result};
use simple_fs::{SPath, ensure_dir};

pub enum InstallResponse {
	Installed(InstalledPack),
	UpToDate(InstalledPack),
}

pub struct InstalledPack {
	pub pack_toml: PackToml,
	pub path: SPath,
	#[allow(unused)]
	pub size: usize,
	pub zip_size: usize,
}

/// Install a `file.aipack` into the .aipack-base/pack/installed directory
///
/// IMPORTANT: Right now, very prelimealy. Should do the following:
///
/// TODO:
/// - Check for an existing installed pack.
/// - If an already installed pack has a semver greater than the new one,
///   return an error so that the caller can handle it with a prompt, and then provide a force flag, for example.
/// - Probably need to remove the existing pack files; otherwise, some leftover files can be an issue.
///
/// Returns the InstalledPack with information about the installed pack.
pub async fn install_pack(dir_context: &DirContext, pack_uri: &str, force: bool) -> Result<InstallResponse> {
	let pack_uri = PackUri::parse(pack_uri);

	// Get the aipack file path, downloading if needed
	let (aipack_zipped_file, pack_uri) = match pack_uri {
		pack_uri @ PackUri::RepoPack(_) => support::download_from_repo(dir_context, pack_uri).await?,
		pack_uri @ PackUri::LocalPath(_) => support::resolve_local_path(dir_context, pack_uri)?,
		pack_uri @ PackUri::HttpLink(_) => support::download_pack(dir_context, pack_uri).await?,
	};

	// Validate file exists and has correct extension
	support::validate_aipack_file(&aipack_zipped_file, &pack_uri.to_string())?;

	// Get the zip file size
	let zip_size = support::get_file_size(&aipack_zipped_file, &pack_uri.to_string())?;

	// Common installation steps for both local and remote files
	let mut install_res = install_aipack_file(dir_context, &aipack_zipped_file, &pack_uri, force)?;

	match install_res {
		InstallResponse::Installed(ref mut p) | InstallResponse::UpToDate(ref mut p) => {
			p.zip_size = zip_size;
		}
	}

	// If the file was downloaded (RepoPack or HttpLink), trash the temporary file
	if matches!(pack_uri, PackUri::RepoPack(_) | PackUri::HttpLink(_)) {
		safer_trash_file(&aipack_zipped_file, Some(DeleteCheck::CONTAINS_AIPACK_BASE))?;
	}

	Ok(install_res)
}

/// Common installation logic for both local and remote aipack files
/// Return the InstalledPack containing pack information and installation details
fn install_aipack_file(
	dir_context: &DirContext,
	aipack_zipped_file: &SPath,
	pack_uri: &PackUri,
	force: bool,
) -> Result<InstallResponse> {
	// -- Get the aipack base pack install dir
	// This is the pack base dir and now, we need ot add `namespace/pack_name`
	let pack_installed_dir = dir_context.aipack_paths().get_base_pack_installed_dir()?;

	// Now, we automatically create, so we do not require it to be init-base
	ensure_dir(&pack_installed_dir)?;

	// Note: This should not happen, as it should have failed in the ensure_dir above.
	//       Howeer, for now,
	if !pack_installed_dir.exists() {
		return Err(Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!(
				"aipack base directory '{pack_installed_dir}' not found.\n   recommendation: Run 'aip init'"
			),
		});
	}

	// -- Extract the pack.toml from zip and validate
	let new_pack_toml = support::extract_pack_toml_from_pack_file(aipack_zipped_file)?;

	// NEW: Validate prerelease format for installation
	support::validate_version_for_install(&new_pack_toml.version)?;

	// -- Check if a pack with the same namespace/name is already installed
	let potential_existing_path = pack_installed_dir.join(&new_pack_toml.namespace).join(&new_pack_toml.name);

	if potential_existing_path.exists() && !force {
		let existing_pack_toml_path = potential_existing_path.join("pack.toml");

		// Try to get the existing pack toml (if it fails, we treat it as 0.0.0 and update)
		let existing_pack_toml = if existing_pack_toml_path.exists() {
			let content = std::fs::read_to_string(existing_pack_toml_path.path()).ok();
			content.and_then(|c| parse_validate_pack_toml(&c, existing_pack_toml_path.as_str()).ok())
		} else {
			None
		};

		if let Some(existing_pack_toml) = existing_pack_toml {
			let ord = support::validate_version_update(&existing_pack_toml.version, &new_pack_toml.version)?;
			match ord {
				std::cmp::Ordering::Equal => {
					return Ok(InstallResponse::UpToDate(InstalledPack {
						pack_toml: existing_pack_toml,
						path: potential_existing_path,
						size: 0,
						zip_size: 0,
					}));
				}
				std::cmp::Ordering::Less => {
					return Err(Error::InstallFailInstalledVersionAbove {
						installed_version: existing_pack_toml.version,
						new_version: new_pack_toml.version,
					});
				}
				std::cmp::Ordering::Greater => {}
			}
		}
	}

	// If we've gotten here, either there's no existing pack or the new version is greater than or equal to the installed version
	let pack_target_dir = pack_installed_dir.join(&new_pack_toml.namespace).join(&new_pack_toml.name);

	// If the directory exists, remove it first to ensure clean installation
	if pack_target_dir.exists() {
		safer_trash_dir(&pack_target_dir, Some(DeleteCheck::CONTAINS_AIPACK_BASE)).map_err(|e| {
			Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause: format!("Failed to trash existing pack directory: {e}"),
			}
		})?;
	}

	zip::unzip_file(aipack_zipped_file, &pack_target_dir).map_err(|e| Error::FailToInstall {
		aipack_ref: pack_uri.to_string(),
		cause: format!("Failed to unzip pack: {e}"),
	})?;

	// Calculate the size of the installed pack
	let size = support::calculate_directory_size(&pack_target_dir)?;

	Ok(InstallResponse::Installed(InstalledPack {
		pack_toml: new_pack_toml,
		path: pack_target_dir,
		size,
		zip_size: 0, // This will be populated by the caller
	}))
}

// region:    --- Tests

#[cfg(test)]
#[path = "../../_tests/tests_installer_impl.rs"]
mod tests;

// endregion: --- Tests
