//! The command executor.
//! Will create it's own queue and listen to ExecCommand events.

use crate::agent::Agent;
use crate::exec::event_action::ExecActionEvent;
use crate::exec::support::open_vscode;
use crate::exec::{ExecStatusEvent, RunRedoCtx, exec_install, exec_list, exec_new, exec_pack, exec_run, exec_run_redo};
use crate::hub::get_hub;
use crate::init::{init_base, init_wks};
use crate::runtime::Runtime;
use crate::{Error, Result};
use derive_more::derive::From;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};

pub struct Executor {
	/// The receiver that this executor will itreate on "start"
	action_rx: Receiver<ExecActionEvent>,
	/// Sender that gets cloned for parts that want to send events
	action_sender: ExecutorSender,

	/// For now, the executor keep the last redoCtx state
	/// Note: This might change to a stack, not sure yet.
	///       For the current feature, this is enough.
	current_redo_ctx: Option<RedoCtx>,
}

/// Contructor
impl Executor {
	pub fn new() -> Self {
		let (tx, rx) = channel(100);
		Executor {
			action_rx: rx,
			action_sender: ExecutorSender::new(tx),
			current_redo_ctx: None,
		}
	}
}

/// Getter
impl Executor {
	pub fn sender(&self) -> ExecutorSender {
		self.action_sender.clone()
	}

	/// Return the latest agent file_path that was executed
	fn get_agent_file_path(&self) -> Option<&str> {
		Some(self.current_redo_ctx.as_ref()?.get_agent()?.file_path())
	}
}

/// Runner
impl Executor {
	pub async fn start(&mut self) -> Result<()> {
		let hub = get_hub();

		loop {
			let Some(cmd) = self.action_rx.recv().await else {
				println!("!!!! Aipack Executor: Channel closed");
				break;
			};

			hub.publish(ExecStatusEvent::StartExec).await;

			match cmd {
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
					self.current_redo_ctx = Some(redo.into());
					hub.publish(ExecStatusEvent::RunEnd).await;
				}

				// -- Interactive Events
				ExecActionEvent::Redo => {
					let Some(redo_ctx) = self.current_redo_ctx.as_ref() else {
						hub.publish(Error::custom("No redo available to be performed")).await;
						continue;
					};

					hub.publish(ExecStatusEvent::RunStart).await;
					match redo_ctx {
						RedoCtx::RunRedoCtx(redo_ctx) => {
							// if sucessul, we recapture the redo_ctx to have the latest agent.
							if let Some(redo_ctx) = exec_run_redo(redo_ctx).await {
								self.current_redo_ctx = Some(redo_ctx.into())
							}
						}
					}
					hub.publish(ExecStatusEvent::RunEnd).await;
				}

				ExecActionEvent::OpenAgent => {
					//
					if let Some(agent_file_path) = self.get_agent_file_path() {
						open_vscode(agent_file_path).await
					}
				}

				// -- Agent Commands
				#[allow(unused)]
				ExecActionEvent::RunAgent(run_agent_params) => {
					//
					todo!()
				}
			}

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
		let event_name: &'static str = (&event).into();
		if let Err(err) = self.tx.send(event).await {
			get_hub()
				.publish(Error::cc(format!("Fail to send action event {}", event_name), err))
				.await;
		};
	}
}

// endregion: --- ExecutorSender
