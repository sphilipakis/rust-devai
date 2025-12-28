use crate::Result;
use crate::exec::assets::{self, ZFile};

// region:    --- Workspace ZFiles
pub fn extract_workspace_config_toml_zfile() -> Result<ZFile> {
	extract_workspace_zfile("config.toml")
}

pub fn extract_workspace_zfile(path: &str) -> Result<ZFile> {
	assets::extract_zfile("workspace", path)
}

#[allow(unused)]
pub fn extract_workspace_pack_file_paths() -> Result<Vec<String>> {
	list_workspace_file_paths_start_with("pack")
}

pub fn list_workspace_file_paths_start_with(prefix: &str) -> Result<Vec<String>> {
	assets::list_file_paths_start_with("workspace", prefix)
}

// endregion: --- Workspace ZFiles

// region:    --- Template ZFiles

pub fn extract_template_pack_toml_zfile() -> Result<ZFile> {
	assets::extract_template_zfile("pack.toml")
}

// endregion: --- Template ZFiles

// region:    --- Base ZFiles

pub fn extract_base_config_default_toml_zfile() -> Result<ZFile> {
	extract_base_zfile("config-default.toml")
}

pub fn extract_base_config_user_toml_zfile() -> Result<ZFile> {
	extract_base_zfile("config-user.toml")
}

pub fn extract_base_pack_installed_file_paths() -> Result<Vec<String>> {
	list_base_file_paths_start_with("pack/installed")
}

pub fn extract_base_pack_custom_file_paths() -> Result<Vec<String>> {
	list_base_file_paths_start_with("pack/custom")
}

fn extract_base_zfile(path: &str) -> Result<ZFile> {
	assets::extract_zfile("base", path)
}

fn list_base_file_paths_start_with(prefix: &str) -> Result<Vec<String>> {
	assets::list_file_paths_start_with("base", prefix)
}

// endregion: --- Base ZFiles

// region:    --- Setup Files

pub fn extract_setup_aip_env_sh_zfile() -> Result<ZFile> {
	assets::extract_zfile("_setup", "aip-env")
}

// endregion: --- Setup Files
