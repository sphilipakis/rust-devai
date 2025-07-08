use super::tui_loop::run_ui_loop;
use crate::Result;
use crate::event::{Tx, new_channel};
use crate::exec::cli::CliArgs;
use crate::exec::{ExecActionEvent, ExecutorTx};
use crate::hub::get_hub;
use crate::store::ModelManager;
use crate::tui::event::AppEvent;
use crate::tui::term_reader::run_term_read;
use derive_more::{Deref, From};
use ratatui::DefaultTerminal;

pub async fn start_tui(mm: ModelManager, executor_tx: ExecutorTx, args: CliArgs) -> Result<()> {
	// -- init terminal
	let terminal = ratatui::init();

	let _ = exec_app(terminal, mm, executor_tx, args).await;

	ratatui::restore();

	Ok(())
}

#[derive(Clone, From, Deref)]
pub struct ExitTx(Tx<()>);

#[derive(Clone, From, Deref)]
pub struct AppTx(Tx<AppEvent>);

// Terminal<CrosstermBackend<Stdout>>
async fn exec_app(
	mut terminal: DefaultTerminal,
	mm: ModelManager,
	executor_tx: ExecutorTx,
	args: CliArgs,
) -> Result<()> {
	// -- Exit Channel
	let (exit_tx, exit_rx) = new_channel::<()>("exit_term");
	let exit_tx = ExitTx::from(exit_tx);

	// -- Setup Term
	terminal.clear()?;

	// -- Create AppEvent channels
	let (app_tx, app_rx) = new_channel::<AppEvent>("app_event");
	let app_tx = AppTx::from(app_tx);

	// -- Running the term_reader tasks
	let _tin_read_handle = run_term_read(app_tx.clone())?;

	// -- Running Tui application
	let _tui_handle = run_ui_loop(terminal, mm, executor_tx.clone(), app_rx, app_tx.clone(), exit_tx)?;

	// -- Start the hub event and forward to App Event
	let hub_rx = get_hub().take_rx()?;
	tokio::spawn(async move {
		// TODO: handle exceptions in both those cases
		loop {
			let hub_evt = hub_rx.recv().await;

			match hub_evt {
				Ok(hub_evt) => {
					let _ = app_tx.send(hub_evt).await;
				}
				Err(err) => {
					// NOTE: for now, just print and stop (this might be erased)
					eprintln!("Error on tui hub_rx loop. Cause {err}");
					return;
				}
			}
		}
	});

	// -- Exec the first cli_args
	let exec_cmd: ExecActionEvent = args.cmd.into();
	tokio::spawn(async move {
		// TODO: handle exceptions in both those cases
		let _ = executor_tx.send(exec_cmd).await;
	});

	// -- Wait for the exit
	let _ = exit_rx.recv().await;

	Ok(())
}
