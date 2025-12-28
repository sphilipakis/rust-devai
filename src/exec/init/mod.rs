// region:    --- Module

mod init_assets;
mod init_base;
mod init_wks;

pub use init_assets::{extract_setup_aip_env_sh_zfile, extract_template_pack_toml_zfile};
pub use init_base::*;
pub use init_wks::*;

// endregion: --- Module
