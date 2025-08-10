// region:    --- Modules

mod app_state_base;
mod app_state_core;
mod impl_action;
mod impl_fmt;
mod impl_model_state;
mod impl_mouse;
mod impl_run;
mod impl_scroll;
mod impl_sys;
mod state_processor;
mod sys_state;

use app_state_core::*;

pub use app_state_base::*;
pub use state_processor::process_app_state;
pub use sys_state::*;

// endregion: --- Modules
