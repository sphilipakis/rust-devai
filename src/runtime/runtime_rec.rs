use crate::Result;
use crate::hub::HubEvent;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::store::Id;
use crate::store::Stage;
use crate::store::rt_model::{LogBmc, LogForCreate, LogKind, RunBmc, RunForCreate, RunForUpdate};
use crate::support::time::now_unix_time_us;

impl Runtime {
	pub async fn rec_trim(&self) -> Result<usize> {
		let count = self.mm().trim()?;
		Ok(count)
	}
}
/// Rec for all step record (like timestamp and all)
/// All the function that "record" the progress of a Runtime execution
impl Runtime {
	pub async fn rec_start(&self, agent_name: &str, agent_path: &str) -> Result<Id> {
		let hub = get_hub();

		let id = RunBmc::create(
			self.mm(),
			RunForCreate {
				agent_name: Some(agent_name.to_string()),
				agent_path: Some(agent_path.to_string()),
				start: Some(now_unix_time_us().into()),
			},
		)?;

		// -- For V1 terminal
		hub.publish(format!(
			"\n======= RUNNING: {agent_name}\n     Agent path: {agent_path}",
		))
		.await;

		Ok(id)
	}

	pub async fn rec_ba_start(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			ba_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	pub async fn rec_ba_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			ba_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the start of Tasks execution.
	pub async fn rec_tasks_start(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			tasks_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the end of Tasks execution.
	pub async fn rec_tasks_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			tasks_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the start of After All execution.
	pub async fn rec_aa_start(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			aa_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the end of After All execution.
	pub async fn rec_aa_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			aa_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}

	/// Mark the run as completed.
	pub async fn rec_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;
		Ok(())
	}
}

/// Rec for the log
impl Runtime {
	async fn rec_log(
		&self,
		run_id: Id,
		task_id: Option<Id>,
		stage: Option<Stage>,
		msg: impl Into<String>,
		kind: Option<LogKind>,
	) -> Result<()> {
		let msg = msg.into();

		let log_c = LogForCreate {
			run_id,
			task_id,
			stage,
			message: Some(msg.clone()),
			kind,
		};
		LogBmc::create(self.mm(), log_c)?;

		// -- For V1 terminal
		let hub = get_hub();
		if let Some(LogKind::SysInfo) = kind {
			hub.publish(HubEvent::info_short(msg)).await;
		} else {
			hub.publish(msg).await;
		}

		Ok(())
	}

	#[allow(unused)]
	pub async fn rec_log_run(&self, run_id: Id, msg: impl Into<String>, kind: Option<LogKind>) -> Result<()> {
		self.rec_log(run_id, None, None, msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_ba(&self, run_id: Id, msg: impl Into<String>, kind: Option<LogKind>) -> Result<()> {
		self.rec_log(run_id, None, Some(Stage::BeforeAll), msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_data(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		kind: Option<LogKind>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), Some(Stage::Data), msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_ai(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		kind: Option<LogKind>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), Some(Stage::Ai), msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_output(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		kind: Option<LogKind>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), Some(Stage::Output), msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_aa(&self, run_id: Id, msg: impl Into<String>, kind: Option<LogKind>) -> Result<()> {
		self.rec_log(run_id, None, Some(Stage::AfterAll), msg, kind).await
	}
}
