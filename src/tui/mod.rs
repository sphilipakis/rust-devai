// region:    --- Modules

mod app_event_handler;
mod event;
mod styles;
mod support;
mod term_reader;
mod tui_impl;
mod tui_loop;
mod views;

pub use tui_impl::*;
pub use tui_loop::AppState;
pub use views::*;

// endregion: --- Modules
