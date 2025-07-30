//! Base module for the script engine.
//!
//! NOTE: At this point, Lua is the only planned scripting language for aipack.
//!       It is small, simple, relatively well-known, efficient, and in many ways was made for these kinds of use cases.
//!

// region:    --- Modules

mod aip_modules;
mod error_lua_support;
mod support;

mod aipack_custom;
mod lua_engine;
mod lua_json;
mod lua_uc;
mod lua_value_ext;

pub use aipack_custom::*;
pub use lua_engine::*;
pub use lua_json::*;
pub use lua_value_ext::LuaValueExt;

#[cfg(test)] // Needed for test only (beside this script module)
pub use support::process_lua_eval_result;

// endregion: --- Modules

const DEFAULT_MARKERS: &(&str, &str) = &("<<START>>", "<<END>>");
