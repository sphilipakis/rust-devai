use crate::Result;
use crate::hub::get_hub;
use crate::model::{Id, ModelManager, RunStep, Stage};
use crate::model::{LogBmc, LogForCreate, LogKind};
use crate::runtime::Runtime;
use derive_more::From;

#[derive(Debug, From)]
pub struct RtLog<'a> {
	runtime: &'a Runtime,
}

/// Constructor & core getters
impl<'a> RtLog<'a> {
	pub fn new(runtime: &'a Runtime) -> RtLog<'a> {
		Self { runtime }
	}

	fn mm(&self) -> &ModelManager {
		self.runtime.mm()
	}
}

/// Core logs
impl<'a> RtLog<'a> {
	pub(super) async fn rec_log(
		&self,
		run_id: Id,
		task_id: Option<Id>,
		step: Option<RunStep>,
		stage: Option<Stage>,
		msg: impl Into<String>,
		kind: Option<LogKind>,
	) -> Result<()> {
		let msg = msg.into();

		let log_c = LogForCreate {
			run_id,
			task_id,
			step,
			stage,
			message: Some(msg.clone()),
			kind,
		};
		LogBmc::create(self.mm(), log_c)?;

		// -- For V1 terminal
		let hub = get_hub();
		hub.publish(msg).await;
		// if let Some(LogKing::SysInfo) = kind {
		// 	hub.publish(HubEvent::info_short(msg)).await;
		// } else {
		// 	hub.publish(msg).await;
		// }

		Ok(())
	}

	pub(super) async fn rec_log_no_msg(
		&self,
		run_id: Id,
		task_id: Option<Id>,
		step: Option<RunStep>,
		stage: Option<Stage>,
		kind: Option<LogKind>,
	) -> Result<()> {
		let log_c = LogForCreate {
			run_id,
			task_id,
			step,
			stage,
			message: None,
			kind,
		};
		LogBmc::create(self.mm(), log_c)?;

		// -- For V1 terminal
		// This is a new log, no legacy equivalent.

		Ok(())
	}
}

impl<'a> RtLog<'a> {
	pub async fn rec_log_run(&self, run_id: Id, msg: impl Into<String>, level: Option<LogKind>) -> Result<()> {
		self.rec_log(run_id, None, None, None, msg, level).await
	}

	pub async fn rec_log_ba(&self, run_id: Id, msg: impl Into<String>, level: Option<LogKind>) -> Result<()> {
		self.rec_log(run_id, None, None, Some(Stage::BeforeAll), msg, level).await
	}

	pub async fn rec_log_data(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		level: Option<LogKind>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), None, Some(Stage::Data), msg, level).await
	}

	pub async fn rec_log_ai(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		level: Option<LogKind>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), None, Some(Stage::Ai), msg, level).await
	}

	pub async fn rec_log_output(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		level: Option<LogKind>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), None, Some(Stage::Output), msg, level).await
	}

	pub async fn rec_log_aa(&self, run_id: Id, msg: impl Into<String>, level: Option<LogKind>) -> Result<()> {
		self.rec_log(run_id, None, None, Some(Stage::AfterAll), msg, level).await
	}
}
