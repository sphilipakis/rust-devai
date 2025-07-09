// region:    --- Modules

mod app_state;
mod app_state_ui;
mod state_processor;
mod tui_impl;
mod tui_loop;

// Note: Expose only a subset
pub use app_state::AppState;
pub use tui_impl::{AppTx, ExitTx, start_tui};

// endregion: --- Modules
