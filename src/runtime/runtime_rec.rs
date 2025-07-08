use crate::Result;
use crate::hub::HubEvent;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::store::Id;
use crate::store::RunStep;
use crate::store::Stage;
use crate::store::rt_model::TaskBmc;
use crate::store::rt_model::TaskForCreate;
use crate::store::rt_model::TaskForUpdate;
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
	pub async fn step_start(&self, agent_name: &str, agent_path: &str) -> Result<Id> {
		let hub = get_hub();

		// -- Create Run
		let run_id = RunBmc::create(
			self.mm(),
			RunForCreate {
				agent_name: Some(agent_name.to_string()),
				agent_path: Some(agent_path.to_string()),
				start: Some(now_unix_time_us().into()),
			},
		)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::Start), None, Some(LogLevel::RunStep))
			.await?;

		// -- For V1 terminal
		hub.publish(format!(
			"\n======= RUNNING: {agent_name}\n     Agent path: {agent_path}",
		))
		.await;

		Ok(run_id)
	}

	pub async fn step_ba_start(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			ba_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::BaStart), None, Some(LogLevel::RunStep))
			.await?;

		Ok(())
	}

	pub async fn step_ba_end(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			ba_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::BaEnd), None, Some(LogLevel::RunStep))
			.await?;

		Ok(())
	}

	/// Mark the start of Tasks execution.
	pub async fn step_tasks_start(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			tasks_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::TasksStart), None, Some(LogLevel::RunStep))
			.await?;

		Ok(())
	}

	/// Mark the end of Tasks execution.
	pub async fn step_tasks_end(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			tasks_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::TasksEnd), None, Some(LogLevel::RunStep))
			.await?;

		Ok(())
	}

	pub async fn step_task_start(&self, run_id: Id, idx: usize) -> Result<Id> {
		// -- Create Task
		let task_c = TaskForCreate {
			run_id,
			start: now_unix_time_us().into(),
			idx: idx as i64,
			label: None,
		};
		let id = TaskBmc::create(self.mm(), task_c)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(id),
			Some(RunStep::TaskStart),
			None,
			Some(LogLevel::RunStep),
		)
		.await?;

		Ok(id)
	}

	pub async fn step_task_data_start(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			data_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskDataStart),
			None,
			Some(LogLevel::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_data_end(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			data_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskDataEnd),
			None,
			Some(LogLevel::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_ai_start(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			ai_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskAiStart),
			None,
			Some(LogLevel::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_ai_end(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			ai_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskAiEnd),
			None,
			Some(LogLevel::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_output_start(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			output_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskOutputStart),
			None,
			Some(LogLevel::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_output_end(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			output_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskOutputEnd),
			None,
			Some(LogLevel::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_end(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task
		let task_u = TaskForUpdate {
			end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskEnd),
			None,
			Some(LogLevel::RunStep),
		)
		.await?;

		Ok(())
	}

	/// Mark the start of After All execution.
	pub async fn step_aa_start(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			aa_start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::AaStart), None, Some(LogLevel::RunStep))
			.await?;

		Ok(())
	}

	/// Mark the end of After All execution.
	pub async fn step_aa_end(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			aa_end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::AaEnd), None, Some(LogLevel::RunStep))
			.await?;

		Ok(())
	}

	/// Mark the run as completed.
	pub async fn step_end(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			end: Some(now_unix_time_us().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::End), None, Some(LogLevel::RunStep))
			.await?;

		Ok(())
	}
}

/// Update model
impl Runtime {
	pub async fn update_run_model(&self, run_id: Id, model_name: &str) -> Result<()> {
		let run_u = RunForUpdate {
			model: Some(model_name.to_string()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		Ok(())
	}

	pub async fn update_task_model(&self, _run_id: Id, task_id: Id, model_name: &str) -> Result<()> {
		let task_u = TaskForUpdate {
			model: Some(model_name.to_string()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		Ok(())
	}

	pub async fn update_task_cost(&self, run_id: Id, task_id: Id, cost: f64) -> Result<()> {
		// -- Add the cost to the task_u
		let task_u = TaskForUpdate {
			cost: Some(cost),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Update the run total cost
		// NOTE: Here we recompute the total cost rather than doing a simple add to avoid
		//       any race condition
		let tasks = TaskBmc::list_for_run(self.mm(), run_id)?;
		let total_cost: f64 = tasks.iter().filter_map(|t| t.cost).sum();
		let run_u = RunForUpdate {
			total_cost: Some(total_cost),
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
		step: Option<RunStep>,
		stage: Option<Stage>,
		msg: impl Into<String>,
		level: Option<LogLevel>,
	) -> Result<()> {
		let msg = msg.into();

		let log_c = LogForCreate {
			run_id,
			task_id,
			step,
			stage,
			message: Some(msg.clone()),
			level,
		};
		LogBmc::create(self.mm(), log_c)?;

		// -- For V1 terminal
		let hub = get_hub();
		if let Some(LogLevel::SysInfo) = level {
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
		level: Option<LogLevel>,
	) -> Result<()> {
		let log_c = LogForCreate {
			run_id,
			task_id,
			step,
			stage,
			message: None,
			level,
		};
		LogBmc::create(self.mm(), log_c)?;

		// -- For V1 terminal
		// This is a new log, no legacy equivalent.

		Ok(())
	}

	pub async fn rec_log_run(&self, run_id: Id, msg: impl Into<String>, level: Option<LogLevel>) -> Result<()> {
		self.rec_log(run_id, None, None, None, msg, level).await
	}

	pub async fn rec_log_ba(&self, run_id: Id, msg: impl Into<String>, level: Option<LogLevel>) -> Result<()> {
		self.rec_log(run_id, None, None, Some(Stage::BeforeAll), msg, level).await
	}

	pub async fn rec_log_data(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		level: Option<LogLevel>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), None, Some(Stage::Data), msg, level).await
	}

	pub async fn rec_log_ai(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		level: Option<LogLevel>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), None, Some(Stage::Ai), msg, level).await
	}

	pub async fn rec_log_output(
		&self,
		run_id: Id,
		task_id: Id,
		msg: impl Into<String>,
		level: Option<LogLevel>,
	) -> Result<()> {
		self.rec_log(run_id, Some(task_id), None, Some(Stage::Output), msg, level).await
	}

	pub async fn rec_log_aa(&self, run_id: Id, msg: impl Into<String>, level: Option<LogLevel>) -> Result<()> {
		self.rec_log(run_id, None, None, Some(Stage::AfterAll), msg, level).await
	}
}
