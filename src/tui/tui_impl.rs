use super::event::{AppEvent, LastAppEvent};
use super::term_reader::run_term_read;
use crate::Result;
use crate::event::{Rx, Tx, new_channel};
use crate::exec::cli::CliArgs;
use crate::exec::{ExecActionEvent, ExecutorTx};
use crate::hub::get_hub;
use crate::store::ModelManager;
use crate::tui::app_event_handler::handle_app_event;
use crate::tui::app_state::AppState;
use crate::tui::{MainView, SumView};
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use derive_more::{Deref, From};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{Receiver, channel};
use tokio::task::JoinHandle;
use tracing::{debug, error};

pub async fn start_tui(mm: ModelManager, executor_tx: ExecutorTx, args: CliArgs) -> Result<()> {
	// -- init terminal
	let terminal = ratatui::init();

	let _ = exec_app(terminal, mm, executor_tx, args).await;

	ratatui::restore();

	Ok(())
}

#[derive(Clone, From, Deref)]
pub(super) struct ExitTx(Tx<()>);

#[derive(Clone, From, Deref)]
pub(super) struct AppTx(Tx<AppEvent>);

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
	let tui_handle = run_ui_loop(terminal, mm, executor_tx.clone(), app_rx, app_tx.clone(), exit_tx)?;

	// -- Start the hub event and forward to App Event
	let hub_rx = get_hub().take_rx()?;
	tokio::spawn(async move {
		// TODO: handle exceptions in both those cases
		loop {
			let hub_evt = hub_rx.recv().await;

			debug!("HUB LOOP - {hub_evt:?}");

			match hub_evt {
				Ok(hub_evt) => {
					app_tx.send(hub_evt).await;
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
	exit_rx.recv().await;

	Ok(())
}

fn run_ui_loop(
	mut terminal: DefaultTerminal,
	mm: ModelManager,
	executor_tx: ExecutorTx,
	mut app_rx: Rx<AppEvent>,
	app_tx: AppTx,
	exit_tx: ExitTx,
) -> Result<JoinHandle<()>> {
	let mut app_state = AppState::default();

	let handle = tokio::spawn(async move {
		let mut last_event: LastAppEvent = LastAppEvent::default();

		loop {
			// -- Draw
			let _ = terminal_draw(&mut terminal, last_event.take(), &mut app_state, &mm);

			// -- Get Next App Event
			let app_event = match app_rx.recv().await {
				Ok(r) => r,
				Err(err) => {
					error!("UI LOOP ERROR. Cause: {err}");
					continue;
				}
			};
			debug!("->> run_ui_loop AppEvent: {app_event:?}");

			handle_app_event(&mut terminal, &mm, &executor_tx, &app_tx, &exit_tx, &app_event).await;

			last_event = app_event.into();
		}
	});
	Ok(handle)
}

fn terminal_draw(
	terminal: &mut DefaultTerminal,
	last_event: LastAppEvent,
	app_state: &mut AppState,
	mm: &ModelManager,
) -> Result<()> {
	terminal.draw(|frame| {
		let area = frame.area();

		// -- Add background
		let bkg = Block::new().on_black();
		frame.render_widget(bkg, area);

		// -- Layout
		let [header_a, main_a, action_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![Constraint::Length(2), Constraint::Fill(1), Constraint::Length(1)])
			.spacing(1)
			.areas(frame.area());

		// -- Add header
		let sum_v = SumView {};
		frame.render_stateful_widget(sum_v, header_a, app_state.mut_sum_state());

		// -- Add main
		let run_v = MainView::new(mm.clone(), last_event.clone());
		frame.render_stateful_widget(run_v, main_a, app_state.mut_run_state());

		// -- Add action
		//... todo
	})?;

	Ok(())
}
