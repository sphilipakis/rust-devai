// region:    --- Modules

mod agent;
mod derive_aliases;
mod dir_context;
mod error;
mod event;
mod exec;
mod hub;
mod model;
mod run;
mod runtime;
mod script;
mod support;
mod term;
mod tui;
mod tui_v1;
mod types;

#[cfg(test)]
mod _test_support;

use crate::exec::Executor;
use crate::exec::cli::CliArgs;
use crate::hub::{HubEvent, get_hub};
use crate::model::OnceModelManager;
use crate::tui_v1::TuiAppV1;
use clap::{Parser, crate_version};
use derive_aliases::*;
use error::{Error, Result};
use tracing_appender::rolling::never;
use tracing_subscriber::EnvFilter;

pub static VERSION: &str = crate_version!();

// endregion: --- Modules

const DEBUG_LOG: bool = false;

#[tokio::main]
async fn main() -> Result<()> {
	// -- Command arguments
	let args = CliArgs::parse(); // Will fail early, but that’s okay.

	// -- Setup debug tracing_subscriber
	// NOTE: need to keep the handle, otherwise dropped, and nothing get added to the file
	let _tracing_guard = if DEBUG_LOG {
		// Create a file appender (will write all logs to ".tmp.log" in the current dir)
		let file_appender = never(".aip-debug-log", "file.log");
		let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

		// Set up the subscriber with the file writer and log level
		tracing_subscriber::fmt()
			.with_writer(non_blocking)
			.with_env_filter(EnvFilter::new("aip=debug"))
			.without_time()
			.with_ansi(false)
			.init();
		// }
		Some(_guard)
	} else {
		None
	};

	// -- The OnceModelManager
	// This way, ModelManager is only created when needed
	let once_mm = OnceModelManager;

	// -- Start executor
	let executor = Executor::new(once_mm);
	let exec_tx = executor.sender();

	// TODO: Probably want to move the spawn inside executor.start
	tokio::spawn(async move {
		// NOTE: This will consume the excecutor (make sure to get exec_sender before start)
		if let Err(err) = executor.start().await {
			let hub = get_hub();
			hub.publish(HubEvent::Error { error: err.into() }).await;
			hub.publish(HubEvent::Quit).await;
		}
	});

	// -- Start UI
	// NOTE: For now, if interactive, we go to new TUI
	//       Otherwise, if non interactive, we go to v1
	if args.cmd.is_interactive() && args.cmd.is_tui() {
		let mm = once_mm.get().await?;
		tui::start_tui(mm, exec_tx, args).await?;
	} else {
		let tui_v1 = TuiAppV1::new(exec_tx);
		// This will wait until all done
		tui_v1.start_with_args(args).await?;
	}

	// -- End
	// Tokio wait for 100ms
	// Note: This will allow the hub message to drain.
	//       This is a short-term trick before we get the whole TUI app.
	// Note: Probably not needed now.
	tokio::time::sleep(std::time::Duration::from_millis(100)).await;
	//println!("\n---- Until next time, happy coding! ----");

	Ok(())
}
