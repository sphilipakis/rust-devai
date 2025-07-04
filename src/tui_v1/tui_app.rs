use crate::Result;
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

/// Starter
impl TuiAppV1 {
	/// Start the app with arg
	pub async fn start_with_args(self, cli_args: CliArgs) -> Result<()> {
		let hub_rx_for_exit = get_hub().subscriber();

		let interactive = cli_args.cmd.is_interactive();

		// -- Start the application (very rudementary "cli UI for now")
		let in_reader = self.start_app(interactive)?;

		// -- Exec the first cli_args
		self.exec_cli_args(cli_args)?;
		// NOTE: for now, we wait unitl the exec is done.
		// let done_rx = self.exec_cli_args(cli_args)?;
		// done_rx.await;

		// -- Wait for the exit
		self.wait_for_exit(hub_rx_for_exit, interactive).await?;

		// -- Make sure to cloase the in_reader if one to restore states
		if let Some(in_reader) = in_reader {
			in_reader.close()
		}

		Ok(())
	}

	/// Very rundemetary app for now, will become full Ratatui app
	/// - It starts the handle_hub_event which is mostly for display
	/// - And starts the handle_in_event to react to user input
	///   - The handle_in_event might return a InReader so that it can be correctly closed on app quit
	fn start_app(&self, interactive: bool) -> Result<Option<InReader>> {
		// -- Will handle the stdout
		self.handle_hub_event(interactive);

		// -- When interactive, handle the stdin
		let in_reader = self.handle_in_event(interactive);

		Ok(in_reader)
	}
}

/// In and Out handlers
impl TuiAppV1 {
	fn handle_in_event(&self, interactive: bool) -> Option<InReader> {
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
	fn handle_hub_event(&self, interactive: bool) {
		let exec_tx = self.executor_tx();

		tokio::spawn(async move {
			let mut rx = get_hub().subscriber();

			loop {
				let evt_res = rx.recv().await;
				match evt_res {
					Ok(event) => {
						if let Err(err) = handle_hub_event(event, &exec_tx, interactive).await {
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
	}
}

/// Lifecyle private functions
impl TuiAppV1 {
	/// Execute the initial cli_args
	///
	/// Returns:
	///
	/// - The oneshot that will be executed after the executor_tx.send
	///
	/// Note: This function is designed to spawn it's on work and return the oneshot described above,
	///       so that it does not block the async caller.
	fn exec_cli_args(&self, cli_args: CliArgs) -> Result<oneshot::Receiver<()>> {
		let exec_cmd: ExecActionEvent = cli_args.cmd.into();
		let executor_tx = self.executor_tx();

		let (done_tx, done_rx) = oneshot::channel();
		tokio::spawn(async move {
			// TODO: handle exceptions in both those cases
			let _ = executor_tx.send(exec_cmd).await;
			let _ = done_tx.send(());
		});

		Ok(done_rx)
	}

	/// Wait for the exit
	/// - When interative mode, wait for HubEvent::Quit
	/// - When not intractive, the first HubEvent::Executor(ExecEvent::End) will end
	async fn wait_for_exit(&self, mut hub_rx: Receiver<HubEvent>, interactive: bool) -> Result<()> {
		loop {
			if let Ok(hub_event) = hub_rx.recv().await {
				match (hub_event, interactive) {
					(HubEvent::Quit, _) => break,
					(HubEvent::Executor(ExecStatusEvent::EndExec), false) => break,
					_ => (),
				}
			}
		}

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
