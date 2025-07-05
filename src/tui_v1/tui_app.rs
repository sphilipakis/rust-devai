use crate::Result;
use crate::event::{Tx, new_channel};
use crate::exec::cli::CliArgs;
use crate::exec::{ExecActionEvent, ExecStatusEvent, ExecutorTx};
use crate::hub::{HubEvent, get_hub};
use crate::term::safer_println;
use crate::tui_v1::hub_event_handler::handle_hub_event;
use crate::tui_v1::in_reader::InReader;
use crossterm::cursor::MoveUp;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use derive_more::{Deref, From};
use tokio::sync::broadcast::Receiver;
use tokio::sync::oneshot;

/// Note: Right now the quick channel is a watch, but might be better to be a mpsc.
#[derive(Debug)]
pub struct TuiAppV1 {
	executor_tx: ExecutorTx,
}

/// Constructor
impl TuiAppV1 {
	pub fn new(executor_tx: ExecutorTx) -> Self {
		Self { executor_tx }
	}
}

/// Getters
impl TuiAppV1 {
	fn executor_tx(&self) -> ExecutorTx {
		self.executor_tx.clone()
	}
}

#[derive(Clone, From, Deref)]
pub struct ExitTx(Tx<()>);

/// Starter
impl TuiAppV1 {
	/// Start the app with arg
	pub async fn start_with_args(self, cli_args: CliArgs) -> Result<()> {
		let (exit_tx, exit_rx) = new_channel::<()>("exit_term");

		// let hub_rx_for_exit = get_hub().subscriber();

		let interactive = cli_args.cmd.is_interactive();

		// -- Start the application (very rudementary "cli UI for now")
		let in_reader = self.start_app(exit_tx.into(), interactive)?;

		// -- Exec the first cli_args
		let exec_cmd: ExecActionEvent = cli_args.cmd.into();
		let executor_tx = self.executor_tx();

		tokio::spawn(async move {
			// TODO: handle exceptions in both those cases
			let _ = executor_tx.send(exec_cmd).await;
		});

		// -- Wait for the exit
		exit_rx.recv().await;

		// -- Make sure to close the in_reader if one to restore states
		if let Some(in_reader) = in_reader {
			in_reader.close()
		}

		Ok(())
	}

	/// Very rundemetary app for now, will become full Ratatui app
	/// - It starts the handle_hub_event which is mostly for display
	/// - And starts the handle_in_event to react to user input
	///   - The handle_in_event might return a InReader so that it can be correctly closed on app quit
	fn start_app(&self, exit_tx: ExitTx, interactive: bool) -> Result<Option<InReader>> {
		// -- Will handle the stdout
		self.run_handle_hub_event(exit_tx.clone(), interactive)?;

		// -- When interactive, handle the stdin
		let in_reader = self.run_handle_in_event(exit_tx, interactive);

		Ok(in_reader)
	}
}

/// In and Out handlers
impl TuiAppV1 {
	fn run_handle_in_event(&self, exit_tx: ExitTx, interactive: bool) -> Option<InReader> {
		if interactive {
			let (in_reader, in_rx) = InReader::new_and_rx();
			in_reader.start();

			let exec_tx = self.executor_tx();

			tokio::spawn(async move {
				let hub = get_hub();
				while let Ok(key_event) = in_rx.recv_async().await {
					match key_event.code {
						// -- Redo
						KeyCode::Char('r') => {
							// clear_last_n_lines(1);
							if key_event.kind == KeyEventKind::Press {
								safer_println("\n-- R pressed - Redo\n", interactive);
								exec_tx.send(ExecActionEvent::Redo).await;
							}
						}

						// -- Quit
						KeyCode::Char('q') => {
							if key_event.kind == KeyEventKind::Press {
								hub.publish(HubEvent::Quit).await
							}
						}

						// -- Open agent
						KeyCode::Char('a') => {
							// clear_last_n_lines(1);
							if key_event.kind == KeyEventKind::Press {
								exec_tx.send(ExecActionEvent::OpenAgent).await;
							}
						}

						// -- Ctrl c
						KeyCode::Char('c')
							if key_event.modifiers.contains(KeyModifiers::CONTROL)
								&& key_event.kind == KeyEventKind::Press =>
						{
							hub.publish(HubEvent::Quit).await;
						}

						_ => (),
					}
				}
			});
			Some(in_reader)
		} else {
			None
		}
	}

	/// The hub events are typically to be displayed to the user one way or another
	/// For now, we just print most of tose event content.
	fn run_handle_hub_event(&self, exit_tx: ExitTx, interactive: bool) -> Result<()> {
		let exec_tx = self.executor_tx();
		let hub_rx = get_hub().take_rx()?;

		tokio::spawn(async move {
			loop {
				let evt_res = hub_rx.recv().await;
				match evt_res {
					Ok(event) => {
						if let Err(err) = handle_hub_event(event, &exec_tx, &exit_tx, interactive).await {
							println!("Tui ERROR while handling handle_hub_event. Cause {err}")
						}
					}
					Err(err) => {
						println!("TuiApp handle_hub_event event error: {err}");
						break;
					}
				}
			}
		});

		Ok(())
	}
}

// region:    --- Support

/// IMPORTANT: Assumes term is in raw mode
/// For now, we keep this code in case. It works, but can be confusing to users.
#[allow(unused)]
fn clear_last_n_lines(n: u16) {
	let mut stdout = std::io::stdout();
	// Move cursor up two lines.
	execute!(stdout, MoveUp(n)).expect("Cannot MoveUp Cursor");

	// Clear the current line (two times to remove two lines).
	for _ in 0..n {
		execute!(stdout, Clear(ClearType::CurrentLine)).expect("Cannot Clear Current Line");
	}
}

// endregion: --- Support
