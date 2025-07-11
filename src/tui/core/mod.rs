// region:    --- Modules

mod app_event_handler;
mod state_processor;
mod sys_state;
mod term_reader;
mod tui_loop;

// -- For cherry
mod app_state;
mod tui_impl;

// -- Public
mod app_state_fmt;
mod direction;

// -- Cherry Flatten
pub use app_state::AppState;
pub use tui_impl::{AppTx, ExitTx, start_tui};

// -- Public flatten
pub use direction::*;

// endregion: --- Modules
