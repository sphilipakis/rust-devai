use crate::Result;
use crate::hub::HubEvent;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::store::Id;
use crate::store::RunStep;
use crate::store::Stage;
use crate::store::rt_model::{LogBmc, LogForCreate, LogLevel, RunBmc, RunForCreate, RunForUpdate};
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

		let run_id = RunBmc::create(
			self.mm(),
			RunForCreate {
				agent_name: Some(agent_name.to_string()),
				agent_path: Some(agent_path.to_string()),
				start: Some(now_unix_time_us().into()),
			},
		)?;

		self.rec_log_no_msg(run_id, None, Some(RunStep::Start), None, Some(LogLevel::SysInfo));

		// -- For V1 terminal
		hub.publish(format!(
			"\n======= RUNNING: {agent_name}\n     Agent path: {agent_path}",
		))
		.await;

		Ok(run_id)
	}

	pub async fn rec_ba_start(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			ba_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		self.rec_log_no_msg(run_id, None, Some(RunStep::BaStart), None, Some(LogLevel::SysInfo));

		Ok(())
	}

	pub async fn rec_ba_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			ba_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		self.rec_log_no_msg(run_id, None, Some(RunStep::BaEnd), None, Some(LogLevel::SysInfo));

		Ok(())
	}

	/// Mark the start of Tasks execution.
	pub async fn rec_tasks_start(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			tasks_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		self.rec_log_no_msg(run_id, None, Some(RunStep::TasksStart), None, Some(LogLevel::SysInfo));

		Ok(())
	}

	/// Mark the end of Tasks execution.
	pub async fn rec_tasks_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			tasks_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		self.rec_log_no_msg(run_id, None, Some(RunStep::TasksEnd), None, Some(LogLevel::SysInfo));

		Ok(())
	}

	/// Mark the start of After All execution.
	pub async fn rec_aa_start(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			aa_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		self.rec_log_no_msg(run_id, None, Some(RunStep::AaStart), None, Some(LogLevel::SysInfo));

		Ok(())
	}

	/// Mark the end of After All execution.
	pub async fn rec_aa_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			aa_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		self.rec_log_no_msg(run_id, None, Some(RunStep::AaEnd), None, Some(LogLevel::SysInfo));

		Ok(())
	}

	/// Mark the run as completed.
	pub async fn rec_end(&self, run_id: Id) -> Result<()> {
		let run_u = RunForUpdate {
			end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		self.rec_log_no_msg(run_id, None, Some(RunStep::End), None, Some(LogLevel::SysInfo));

		Ok(())
	}
}

/// Rec for the log
impl Runtime {
	async fn rec_log(
		&self,
		run_id: Id,
		task_id: Option<Id>,
		step: Option<RunStep>,
		stage: Option<Stage>,
		msg: impl Into<String>,
		kind: Option<LogLevel>,
	) -> Result<()> {
		let msg = msg.into();

		let log_c = LogForCreate {
			run_id,
			task_id,
			step,
			stage,
			message: Some(msg.clone()),
			level: kind,
		};
		LogBmc::create(self.mm(), log_c)?;

		// -- For V1 terminal
		let hub = get_hub();
		if let Some(LogLevel::SysInfo) = kind {
			hub.publish(HubEvent::info_short(msg)).await;
		} else {
			hub.publish(msg).await;
		}

		Ok(())
	}

	async fn rec_log_no_msg(
		&self,
		run_id: Id,
		task_id: Option<Id>,
		step: Option<RunStep>,
		stage: Option<Stage>,
		kind: Option<LogLevel>,
	) -> Result<()> {
		let log_c = LogForCreate {
			run_id,
			task_id,
			step,
			stage,
			message: None,
			level: kind,
		};
		LogBmc::create(self.mm(), log_c)?;

		// -- For V1 terminal
		// This is a new log, no legacy equivalent.

		Ok(())
	}

	#[allow(unused)]
	pub async fn rec_log_run(&self, run_id: Id, msg: impl Into<String>, kind: Option<LogLevel>) -> Result<()> {
		self.rec_log(run_id, None, None, None, msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_ba(&self, run_id: Id, msg: impl Into<String>, kind: Option<LogLevel>) -> Result<()> {
		self.rec_log(run_id, None, None, Some(Stage::BeforeAll), msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_data(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		kind: Option<LogLevel>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), None, Some(Stage::Data), msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_ai(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		kind: Option<LogLevel>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), None, Some(Stage::Ai), msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_output(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		kind: Option<LogLevel>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), None, Some(Stage::Output), msg, kind).await
	}

	#[allow(unused)]
	pub async fn rec_log_aa(&self, run_id: Id, msg: impl Into<String>, kind: Option<LogLevel>) -> Result<()> {
		self.rec_log(run_id, None, None, Some(Stage::AfterAll), msg, kind).await
	}
}
