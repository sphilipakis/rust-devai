//! The command executor.
//! Will create it's own queue and listen to ExecCommand events.

use crate::agent::find_agent;
use crate::event::{CancelTrx, new_cancel_trx};
use crate::exec::event_action::ExecActionEvent;
use crate::exec::exec_cmd_xelf::exec_xelf_update;
use crate::exec::exec_sub_agent::exec_run_sub_agent;
use crate::exec::init::{init_base, init_base_and_dir_context, init_wks};
use crate::exec::{
	ExecStatusEvent,
	exec_check_keys,
	exec_install,
	exec_list,
	exec_new,
	exec_pack,
	exec_run,
	exec_run_redo,
	exec_xelf_setup, // Added import
};
use crate::hub::{HubEvent, get_hub};
use crate::model::OnceModelManager;
use crate::model::{EndState, ErrBmc, ErrForCreate, WorkBmc, WorkForCreate, WorkForUpdate, WorkKind};
use crate::run::{RunQueueExecutor, RunQueueTx, RunRedoCtx};
use crate::runtime::Runtime;
use crate::support::editor;
use crate::support::time::now_micro;
use crate::{Error, Result};
use flume::{Receiver, Sender};
use simple_fs::SPath;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::runtime::Handle;
use tokio::sync::Mutex;

/// The executor executes all actions of the system.
/// There are three types of action sources:
/// - CLI command     - The original command line that performs the first job, e.g., `aip run my-agent`
/// - CLI interactive - When the user interacts with the CLI, e.g., pressing `r` for redo
/// - Agent logic     - When the agent calls some agent action, e.g., `aip.agent.run("my-agent")`
///
/// NOTE: We might want to split that with cmd_executor, rt_executor (for agent). But not sure yet
///
/// Other parts of the system can get the `ExecutorSender` and clone it to communicate with the executor.
///
/// The executor is designed to execute multiple actions at the same time. It keeps some states (currently just the RedoCtx)
/// so that commands like "Redo" can be performed.
pub struct Executor {
	once_mm: OnceModelManager,

	/// The receiver that this executor will itreate on "start"
	action_rx: Receiver<ExecActionEvent>,
	/// Sender that gets cloned for parts that want to send events
	action_sender: ExecutorTx,

	/// For now, the executor keep the last redoCtx state
	/// Note: This might change to a stack, not sure yet.
	///       For the current feature, this is enough.
	current_redo_ctx: Arc<Mutex<Option<RunRedoCtx>>>,

	/// Tracks the number of active execution actions
	/// Used to send StartExec and EndExec events only when needed
	active_actions: Arc<AtomicUsize>,

	cancel_trx: Option<CancelTrx>,

	/// NOT USED YET
	#[allow(unused)]
	run_queue_tx: RunQueueTx,
}

/// Contructor
impl Executor {
	pub fn new(once_mm: OnceModelManager) -> Self {
		let (tx, rx) = flume::unbounded();
		let run_executor = RunQueueExecutor::new();
		let run_queue_tx = run_executor.start();

		let cancel_trx = new_cancel_trx("cancel_run");

		Executor {
			once_mm,
			action_rx: rx,
			action_sender: ExecutorTx::new(tx),
			current_redo_ctx: Default::default(),
			active_actions: Arc::new(AtomicUsize::new(0)),
			cancel_trx: Some(cancel_trx),
			run_queue_tx,
		}
	}
}

/// Getter & Setters
impl Executor {
	pub fn sender(&self) -> ExecutorTx {
		self.action_sender.clone()
	}

	/// Return the latest agent file_path that was executed
	async fn get_agent_file_path(&self) -> Option<SPath> {
		let redo_ctx = self.current_redo_ctx.lock().await;

		redo_ctx.as_ref().map(|r| r.agent()).map(|a| a.file_path()).map(SPath::new)
	}

	async fn set_current_redo_ctx(&self, redo_ctx: RunRedoCtx) {
		let mut guard = self.current_redo_ctx.lock().await;
		*guard = Some(redo_ctx);
	}

	async fn take_current_redo_ctx(&self) -> Option<RunRedoCtx> {
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
		// NOTE: This pattern of Arc itself and clone per action might need to be revisited.
		let executor = Arc::new(self);

		loop {
			let Ok(action) = executor.action_rx.recv_async().await else {
				println!("!!!! Aipack Executor: Channel closed");
				tracing::error!("Aipack Executor: Channel closed");
				break;
			};

			let xt = executor.clone();

			let action_str = action.as_str();
			let is_tui = action.is_tui();
			// Spawn a new async task for each action

			tokio::spawn(async move {
				// -- exec the action
				let res = xt.perform_action(action).await;

				// -- Handle error
				if let Err(err) = res {
					// if tui, then, store it in the db
					if is_tui && let Ok(mm) = xt.once_mm.get().await {
						let msg = format!("Fail to perform action '{action_str}'.\nCause: {err}");
						let _ = ErrBmc::create(
							&mm,
							ErrForCreate {
								stage: None,
								run_id: None,
								task_id: None,
								typ: None,
								content: Some(msg),
							},
						);
					};

					// NOTE: Traditional cli
					get_hub()
						.publish(Error::cc(format!("Fail to perform action '{action_str}'"), err))
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
			ExecActionEvent::CmdNew(new_args) => {
				if let Err(err) = exec_new(new_args, init_wks(None, false).await?).await {
					if matches!(err, Error::UserInterrupted) {
						hub.publish(HubEvent::InfoShort("New agent creation cancelled by user".into()))
							.await;
						hub.publish(HubEvent::Quit).await;
					} else {
						return Err(err);
					}
				}
				hub.publish(HubEvent::Quit).await;
			}
			ExecActionEvent::CmdList(list_args) => {
				exec_list(init_base_and_dir_context(false).await?, list_args).await?
			}

			ExecActionEvent::CmdPack(pack_args) => exec_pack(&pack_args).await?,

			ExecActionEvent::CmdInstall(install_args) => {
				let _ = exec_install(init_base_and_dir_context(false).await?, install_args).await?;
			}

			ExecActionEvent::CmdCheckKeys(args) => {
				// Does not require dir_context or runtime
				exec_check_keys(args).await?;
			}

			ExecActionEvent::CmdXelfSetup(args) => {
				// Does not require dir_context or runtime (for now)
				exec_xelf_setup(args).await?;
			}

			ExecActionEvent::CmdXelfUpdate(args) => {
				// Does not require dir_context or runtime (for now)
				exec_xelf_update(args).await?;
			}

			ExecActionEvent::OpenAgent => {
				//
				if let Some(agent_file_path) = self.get_agent_file_path().await {
					if let Err(err) = editor::open_file_auto(&agent_file_path) {
						hub.publish(Error::cc("Fail to open agent file in editor", err)).await;
					}
				}
			}

			// -- Agent Run Related

			// From the inital command run
			// TODO: Might want to not initialize the workspace here, and let the user know. Not sure.
			// NOTE: This is the Event from the Command line only (when aip.agent.run, the event RunAgent is sent)
			ExecActionEvent::CmdRun(run_args) => {
				hub.publish(ExecStatusEvent::RunStart).await;
				// Here we init base if version changed.
				// This way we make sure doc and all work as expected
				init_base(false).await?;

				let dir_ctx = init_wks(None, false).await?;
				let exec_sender = self.sender();
				let mm = self.once_mm.get().await?;

				// -- Attempt to find agent early to detect missing packs
				let agent_name = run_args.cmd_agent_name.clone();
				let runtime = Runtime::new(
					dir_ctx.clone(),
					exec_sender.clone(),
					mm.clone(),
					self.cancel_trx.clone(),
				)
				.await?;

				let agent_res = find_agent(&agent_name, &runtime, None);

				match agent_res {
					Ok(_agent) => {
						let redo = exec_run(run_args, runtime).await?;
						self.set_current_redo_ctx(redo).await;
					}
					Err(err) => {
						// Check if it's a missing pack candidate
						if agent_name.contains('@') {
							let (pack_ref, _) = agent_name.split_once('/').unwrap_or((&agent_name, ""));

							// Create Work entry for installation
							let work_data = serde_json::json!({ "pack_ref": pack_ref }).to_string();
							let work_id = WorkBmc::create(
								&mm,
								WorkForCreate {
									kind: WorkKind::Install,
									data: Some(work_data),
								},
							)?;

							// Trigger actual install
							let install_args = crate::exec::cli::InstallArgs {
								aipack_ref: pack_ref.to_string(),
							};

							// Mark work as started
							WorkBmc::update(
								&mm,
								work_id,
								WorkForUpdate {
									start: Some(now_micro().into()),
									..Default::default()
								},
							)?;

							match exec_install(dir_ctx, install_args).await {
								Ok(installed_pack) => {
									// Mark work as completed
									WorkBmc::update(
										&mm,
										work_id,
										WorkForUpdate {
											end: Some(now_micro().into()),
											end_state: Some(EndState::Ok),
											message: Some(format!(
												"{}@{} v{}",
												installed_pack.pack_toml.namespace,
												installed_pack.pack_toml.name,
												installed_pack.pack_toml.version
											)),
											..Default::default()
										},
									)?;

									// Retry the run after successful installation
									let redo = exec_run(run_args, runtime).await?;
									self.set_current_redo_ctx(redo).await;
								}
								Err(install_err) => {
									// Mark work as failed
									WorkBmc::update(
										&mm,
										work_id,
										WorkForUpdate {
											end: Some(now_micro().into()),
											end_state: Some(EndState::Err),
											message: Some(install_err.to_string()),
											..Default::default()
										},
									)?;
									hub.publish(install_err).await;
								}
							}
						} else {
							hub.publish(err).await;
						}
					}
				}

				hub.publish(ExecStatusEvent::RunEnd).await;
			}

			// From Redo
			ExecActionEvent::Redo => {
				if let Some(redo_ctx) = self.take_current_redo_ctx().await {
					hub.publish(ExecStatusEvent::RunStart).await;
					// if sucessful, we recapture the redo_ctx to have the latest agent.
					if let Some(redo_ctx) = exec_run_redo(&redo_ctx).await {
						self.set_current_redo_ctx(redo_ctx).await;
					}
					// if fail, we set the old one to make sure it can be retried
					else {
						self.set_current_redo_ctx(redo_ctx).await;
					}
				} else {
					hub.publish(HubEvent::InfoShort("Agent currently running, wait until done.".into()))
						.await;
				}
				hub.publish(ExecStatusEvent::RunEnd).await;
			}

			// From aip.agent.run
			ExecActionEvent::RunSubAgent(run_agent_params) => {
				if let Err(err) = exec_run_sub_agent(run_agent_params).await {
					hub.publish(Error::cc("Fail to run agent", err)).await;
				}
			}

			// Cancel
			ExecActionEvent::CancelRun => {
				if let Some(tx) = self.cancel_trx.as_ref().map(|trx| trx.tx()) {
					tx.cancel();
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

// region:    --- ExecutorSender

/// The Executor Sender is a wrapper over `Sender<ExecActionEvent>` and some domain specific functions
/// It is acquired from the `Executor` with `sender()` or from `Runtime` with `executor_sender()`
#[derive(Debug, Clone)]
pub struct ExecutorTx {
	tx: Sender<ExecActionEvent>,
}

impl ExecutorTx {
	/// Create a new executor sender
	/// Note: This is private to this module as Runtime and others will clone ExecutorSender to get a new one
	///       as they need to point to the same receiver
	fn new(tx: Sender<ExecActionEvent>) -> Self {
		ExecutorTx { tx }
	}

	/// This is preferred send when possible
	pub async fn send(&self, event: ExecActionEvent) {
		let event_str: &'static str = (&event).into();
		if let Err(err) = self.tx.send_async(event).await {
			get_hub()
				.publish_err(format!("Fail to send action event {event_str}"), Some(err))
				.await;
		};
	}

	/// Send the message using flume sync send.
	///
	/// NOTE: This uses the flume synchronous send, which works well in most scenarios.
	///       However, when the queue handle each event in its own spawn, as tthe executor does
	///       this will only received when the previous event is completed (this was the issue with aip_agent run).
	///       This is why we have the send_sync_spawn_and_block below.
	pub fn send_sync(&self, event: ExecActionEvent) {
		let event_str: &'static str = (&event).into();
		if let Err(err) = self.tx.send(event) {
			get_hub().publish_err_sync(format!("Fail to send action event {event_str}"), Some(err));
		}
	}

	/// Use this when sending to the same queue from a sync function
	/// and we want the event to be processed in parallel
	/// (which Executor queue allows because each event processing is its own spawn)
	/// This should be used in all aip_... call when there are.
	/// NOTE: Eventually, we might have another queue for running agent, so aip_agent run might use that other queue
	pub fn send_sync_spawn_and_block(&self, event: ExecActionEvent) -> Result<()> {
		if let Ok(handle) = Handle::try_current() {
			//
			tokio::task::block_in_place(|| {
				handle.block_on(async {
					self.send(event).await;
				})
			});
			Ok(())
		} else {
			Err(Error::custom(
				"Executor Tx send_sync_block_on failed because no current tokio handle",
			))
		}
	}
}

// endregion: --- ExecutorSender
