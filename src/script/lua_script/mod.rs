// region:    --- Modules

mod aip_modules;
mod helpers;
mod lua_engine;
mod lua_json;
mod lua_value_ext;

pub use aip_modules::*;
pub use lua_engine::*;
pub use lua_json::*;
pub use lua_value_ext::*;

#[cfg(test)] // might want to refactor this one
pub use helpers::process_lua_eval_result;

// endregion: --- Modules

const DEFAULT_MARKERS: &(&str, &str) = &("<<START>>", "<<END>>");
