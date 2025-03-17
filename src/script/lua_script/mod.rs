// region:    --- Modules

mod aip_agent;
mod aip_cmd;
mod aip_code;
mod aip_file;
mod aip_flow;
mod aip_git;
mod aip_hbs;
mod aip_html;
mod aip_json;
mod aip_lua;
mod aip_md;
mod aip_path;
mod aip_rust;
mod aip_semver;
mod aip_text;
mod aip_web;
mod helpers;
mod lua_engine;
mod lua_value_ext;

pub use lua_engine::*;
pub use lua_value_ext::*;

pub use helpers::*;

// endregion: --- Modules

const DEFAULT_MARKERS: &(&str, &str) = &("<<START>>", "<<END>>");
