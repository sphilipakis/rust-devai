//! The command executor.
//! Will create it's own queue and listen to ExecCommand events.

use crate::agent::Agent;
use crate::exec::event_action::ExecActionEvent;
use crate::exec::exec_agent_run::exec_run_agent;
use crate::exec::support::open_vscode;
use crate::exec::{ExecStatusEvent, RunRedoCtx, exec_install, exec_list, exec_new, exec_pack, exec_run, exec_run_redo};
use crate::hub::get_hub;
use crate::init::{init_base, init_wks};
use crate::runtime::Runtime;
use crate::{Error, Result};
use derive_more::derive::From;
use flume::{Receiver, Sender};
use simple_fs::SPath;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;

/// The executor executes all actions of the system.
/// There are three types of action sources:
/// - CLI command     - The original command line that performs the first job, e.g., `aip run my-agent`
/// - CLI interactive - When the user interacts with the CLI, e.g., pressing `r` for redo
/// - Agent logic     - When the agent calls some agent action, e.g., `aip.agent.run("my-agent")`
///
/// Other parts of the system can get the `ExecutorSender` and clone it to communicate with the executor.
///
/// The executor is designed to execute multiple actions at the same time. It keeps some states (currently just the RedoCtx)
/// so that commands like "Redo" can be performed.
pub struct Executor {
	/// The receiver that this executor will itreate on "start"
	action_rx: Receiver<ExecActionEvent>,
	/// Sender that gets cloned for parts that want to send events
	action_sender: ExecutorSender,

	/// For now, the executor keep the last redoCtx state
	/// Note: This might change to a stack, not sure yet.
	///       For the current feature, this is enough.
	current_redo_ctx: Arc<Mutex<Option<RedoCtx>>>,

	/// Tracks the number of active execution actions
	/// Used to send StartExec and EndExec events only when needed
	active_actions: Arc<AtomicUsize>,
}

/// Contructor
impl Executor {
	pub fn new() -> Self {
		let (tx, rx) = flume::unbounded();
		Executor {
			action_rx: rx,
			action_sender: ExecutorSender::new(tx),
			current_redo_ctx: Default::default(),
			active_actions: Arc::new(AtomicUsize::new(0)),
		}
	}
}

/// Getter & Setters
impl Executor {
	pub fn sender(&self) -> ExecutorSender {
		self.action_sender.clone()
	}

	/// Return the latest agent file_path that was executed
	async fn get_agent_file_path(&self) -> Option<SPath> {
		let redo_ctx = self.current_redo_ctx.lock().await;
		let path = redo_ctx
			.as_ref()
			.and_then(|r| r.get_agent())
			.map(|a| a.file_path())
			.map(SPath::new);
		path
	}

	async fn set_current_redo_ctx(&self, redo_ctx: RedoCtx) {
		let mut guard = self.current_redo_ctx.lock().await;
		*guard = Some(redo_ctx);
	}

	async fn take_current_redo_ctx(&self) -> Option<RedoCtx> {
		let mut guard = self.current_redo_ctx.lock().await;
		guard.take()
	}

	/// Increment active actions counter and return if this is the first action
	fn increment_actions(&self) -> bool {
		let prev_count = self.active_actions.fetch_add(1, Ordering::SeqCst);
		prev_count == 0
	}

	/// Decrement active actions counter and return if this was the last action
	fn decrement_actions(&self) -> bool {
		let prev_count = self.active_actions.fetch_sub(1, Ordering::SeqCst);
		prev_count == 1
	}
}

/// Runner
impl Executor {
	pub async fn start(self) -> Result<()> {
		let executor = Arc::new(self);

		loop {
			let Ok(action) = executor.action_rx.recv_async().await else {
				println!("!!!! Aipack Executor: Channel closed");
				break;
			};

			let xt = executor.clone();

			let action_str = action.as_str();
			// Spawn a new async task for each action

			tokio::spawn(async move {
				if let Err(err) = xt.perform_action(action).await {
					get_hub()
						.publish(format!("Fail to perform action '{action_str}'. Cause: {err}"))
						.await;
				}
			});
		}

		Ok(())
	}

	async fn perform_action(&self, action: ExecActionEvent) -> Result<()> {
		let hub = get_hub();

		// Only send StartExec if this is the first action
		let is_first_action = self.increment_actions();
		if is_first_action {
			hub.publish(ExecStatusEvent::StartExec).await;
		}

		match action {
			// -- Cli Action Events
			ExecActionEvent::CmdInit(init_args) => {
				init_wks(init_args.path.as_deref(), true).await?;
				init_base(false).await?;
			}
			ExecActionEvent::CmdInitBase => {
				init_base(true).await?;
			}
			// TODO: need to rethink this action
			ExecActionEvent::CmdNewAgent(new_args) => {
				exec_new(new_args, init_wks(None, false).await?).await?;
			}
			ExecActionEvent::CmdList(list_args) => exec_list(init_wks(None, false).await?, list_args).await?,

			ExecActionEvent::CmdPack(pack_args) => exec_pack(&pack_args).await?,

			ExecActionEvent::CmdInstall(install_args) => {
				exec_install(init_wks(None, false).await?, install_args).await?
			}

			ExecActionEvent::CmdRun(run_args) => {
				hub.publish(ExecStatusEvent::RunStart).await;
				let dir_ctx = init_wks(None, false).await?;
				// NOTE: For now, we create the runtime here. But we need to think more about the Runtime / Executor relationship.
				let exec_sender = self.sender();
				let runtime = Runtime::new(dir_ctx, exec_sender)?;
				let redo = exec_run(run_args, runtime).await?;
				self.set_current_redo_ctx(redo.into()).await;
				hub.publish(ExecStatusEvent::RunEnd).await;
			}

			// -- Interactive Events
			ExecActionEvent::Redo => {
				if let Some(redo_ctx) = self.take_current_redo_ctx().await {
					hub.publish(ExecStatusEvent::RunStart).await;
					match redo_ctx {
						RedoCtx::RunRedoCtx(redo_ctx_orig) => {
							// if sucessull, we recapture the redo_ctx to have the latest agent.
							if let Some(redo_ctx) = exec_run_redo(&redo_ctx_orig).await {
								self.set_current_redo_ctx(redo_ctx.into()).await;
							}
							// if fail, we set the old one to make sure it can be retried
							else {
								self.set_current_redo_ctx(redo_ctx_orig.into()).await;
							}
						}
					}
				} else {
					hub.publish(Error::custom("No redo available to be performed")).await;
				}
				hub.publish(ExecStatusEvent::RunEnd).await;
			}

			ExecActionEvent::OpenAgent => {
				//
				if let Some(agent_file_path) = self.get_agent_file_path().await {
					open_vscode(agent_file_path).await
				}
			}

			// -- Agent Commands
			ExecActionEvent::RunAgent(run_agent_params) => {
				if let Err(err) = exec_run_agent(run_agent_params).await {
					hub.publish(Error::cc("Fail to run agent", err)).await;
				}
			}
		}

		// Only send EndExec if this is the last action
		let is_last_action = self.decrement_actions();
		if is_last_action {
			hub.publish(ExecStatusEvent::EndExec).await;
		}

		Ok(())
	}
}

// region:    --- RedoCtx

#[derive(From)]
enum RedoCtx {
	RunRedoCtx(Arc<RunRedoCtx>),
}

impl From<RunRedoCtx> for RedoCtx {
	fn from(run_redo_ctx: RunRedoCtx) -> Self {
		RedoCtx::RunRedoCtx(run_redo_ctx.into())
	}
}

impl RedoCtx {
	pub fn get_agent(&self) -> Option<&Agent> {
		match self {
			RedoCtx::RunRedoCtx(redo_ctx) => Some(redo_ctx.agent()),
		}
	}
}

// endregion: --- RedoCtx

// region:    --- ExecutorSender

/// The Executor Sender is a wrapper over `Sender<ExecActionEvent>` and some domain specific functions
/// It is acquired from the `Executor` with `sender()` or from `Runtime` with `executor_sender()`
#[derive(Debug, Clone)]
pub struct ExecutorSender {
	tx: Sender<ExecActionEvent>,
}

impl ExecutorSender {
	/// Create a new executor sender
	/// Note: This is private to this module as Runtime and others will clone ExecutorSender to get a new one
	///       as they need to point to the same receiver
	fn new(tx: Sender<ExecActionEvent>) -> Self {
		ExecutorSender { tx }
	}

	pub async fn send(&self, event: ExecActionEvent) {
		let event_str: &'static str = (&event).into();
		if let Err(err) = self.tx.send_async(event).await {
			get_hub()
				.publish(Error::cc(format!("Fail to send action event {}", event_str), err))
				.await;
		};
	}
}

// endregion: --- ExecutorSender
