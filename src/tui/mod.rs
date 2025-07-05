// region:    --- Modules

mod app_event_handler;
mod app_state;
mod event;
mod styles;
mod support;
mod term_reader;
mod tui_impl;
mod views;

use app_state::*;

pub use tui_impl::*;
pub use views::*;

// endregion: --- Modules
