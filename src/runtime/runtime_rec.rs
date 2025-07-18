//! This is the Runtime Record function that are called
//! - from the run::run_agent... functions
//! - from the tui2 in some event to save some data (print, save, ...)
//!
//! NOTE:
//! - Here many of the function are async but the probably do not need to be. Might change this later.

use crate::Result;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::store::EndState;
use crate::store::Id;
use crate::store::RunStep;
use crate::store::Stage;
use crate::store::TypedContent;
use crate::store::rt_model::TaskBmc;
use crate::store::rt_model::TaskForCreate;
use crate::store::rt_model::TaskForUpdate;
use crate::store::rt_model::{LogBmc, LogForCreate, LogKind, RunBmc, RunForCreate, RunForUpdate};
use crate::support::time::now_micro;
use serde_json::Value;

impl Runtime {
	pub async fn rec_trim(&self) -> Result<usize> {
		let count = self.mm().trim()?;
		Ok(count)
	}
}

/// Rec for all step record (like timestamp and all)
/// All the function that "record" the progress of a Runtime execution
impl Runtime {
	pub async fn step_run_start(&self, run_id: Id) -> Result<Id> {
		// -- Update start time
		RunBmc::update(
			self.mm(),
			run_id,
			RunForUpdate {
				start: Some(now_micro().into()),
				..Default::default()
			},
		)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::Start), None, Some(LogKind::RunStep))
			.await?;

		Ok(run_id)
	}

	pub async fn step_ba_start(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			ba_start: Some(now_micro().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::BaStart), None, Some(LogKind::RunStep))
			.await?;

		Ok(())
	}

	pub async fn step_ba_end(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			ba_end: Some(now_micro().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::BaEnd), None, Some(LogKind::RunStep))
			.await?;

		Ok(())
	}

	/// Mark the start of Tasks execution.
	pub async fn step_tasks_start(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			tasks_start: Some(now_micro().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::TasksStart), None, Some(LogKind::RunStep))
			.await?;

		Ok(())
	}

	/// Mark the end of Tasks execution.
	pub async fn step_tasks_end(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			tasks_end: Some(now_micro().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::TasksEnd), None, Some(LogKind::RunStep))
			.await?;

		Ok(())
	}

	pub async fn step_task_start(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			start: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskStart),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_data_start(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			data_start: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskDataStart),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_data_end(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			data_end: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskDataEnd),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_ai_start(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			ai_start: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskAiStart),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_ai_gen_start(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			ai_gen_start: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskAiStart),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_ai_gen_end(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			ai_gen_end: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskAiStart),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_ai_end(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			ai_end: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskAiEnd),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_output_start(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			output_start: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskOutputStart),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	pub async fn step_task_output_end(&self, run_id: Id, task_id: Id) -> Result<()> {
		// -- Update Task State
		let task_u = TaskForUpdate {
			output_end: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskOutputEnd),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	/// NOTE: Will update end_state if None
	pub async fn step_task_end_ok(&self, run_id: Id, task_id: Id) -> Result<()> {
		let mm = self.mm();

		let task = TaskBmc::get(mm, task_id)?;

		// Only update end state if none
		let end_state = if task.end_state.is_none() {
			Some(EndState::Ok)
		} else {
			None
		};

		// -- Update Task State
		let task_u = TaskForUpdate {
			end: Some(now_micro().into()),
			end_state,
			..Default::default()
		};
		TaskBmc::update(mm, task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskEnd),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	/// Note will update
	pub async fn step_task_end_err(&self, run_id: Id, task_id: Id, err: &crate::Error) -> Result<()> {
		let mm = self.mm();

		let task = TaskBmc::get(mm, task_id)?;

		// -- if we do not have a end_err_id, we set this one
		// Note: this will se the end_state as well
		if task.end_err_id.is_none() {
			TaskBmc::set_end_error(mm, task_id, None, err)?;
		}

		// -- Update Task State
		let task_u = TaskForUpdate {
			end: Some(now_micro().into()),
			..Default::default()
		};
		TaskBmc::update(mm, task_id, task_u)?;

		// -- Add log line
		self.rec_log_no_msg(
			run_id,
			Some(task_id),
			Some(RunStep::TaskEnd),
			None,
			Some(LogKind::RunStep),
		)
		.await?;

		Ok(())
	}

	/// Mark the start of After All execution.
	pub async fn step_aa_start(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			aa_start: Some(now_micro().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::AaStart), None, Some(LogKind::RunStep))
			.await?;

		Ok(())
	}

	/// Mark the end of After All execution.
	pub async fn step_aa_end(&self, run_id: Id) -> Result<()> {
		// -- Update Run State
		let run_u = RunForUpdate {
			aa_end: Some(now_micro().into()),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::AaEnd), None, Some(LogKind::RunStep))
			.await?;

		Ok(())
	}

	/// Mark the run as completed (end time, end_state)
	pub async fn step_run_end_ok(&self, run_id: Id) -> Result<()> {
		let mm = self.mm();

		let run = RunBmc::get(mm, run_id)?;

		// Only update end state if none
		let end_state = if run.end_state.is_none() {
			Some(EndState::Ok)
		} else {
			None
		};

		// -- Update Run State
		let run_u = RunForUpdate {
			end: Some(now_micro().into()),
			end_state,
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::End), None, Some(LogKind::RunStep))
			.await?;

		Ok(())
	}

	/// Mark the run as ended but with error
	/// NOTE: if the err_id is already there, just do not create a new one.
	///       However, it not there, will create new one.
	pub async fn step_run_end_err(&self, run_id: Id, err: &crate::Error) -> Result<()> {
		let mm = self.mm();

		let run = RunBmc::get(mm, run_id)?;

		// -- if we do not have a end_err_id, we set this one
		// Note: this will se the end_state as well
		if run.end_err_id.is_none() {
			RunBmc::set_end_error(mm, run_id, None, err)?;
		}

		// -- Then set the end time
		// No need to set end_state was done by set_end_error
		let run_u = RunForUpdate {
			end: Some(now_micro().into()),
			..Default::default()
		};
		RunBmc::update(mm, run_id, run_u)?;

		// -- Now update all the tasks of the run
		TaskBmc::cancel_all_not_ended_for_run(mm, run_id)?;

		// -- Add log line
		self.rec_log_no_msg(run_id, None, Some(RunStep::End), None, Some(LogKind::RunStep))
			.await?;

		Ok(())
	}
}

/// Run Create/Update model
impl Runtime {
	pub async fn create_run(&self, agent_name: &str, agent_path: &str) -> Result<Id> {
		let hub = get_hub();

		// -- Create Run
		let run_id = RunBmc::create(
			self.mm(),
			RunForCreate {
				agent_name: Some(agent_name.to_string()),
				agent_path: Some(agent_path.to_string()),
			},
		)?;

		// -- For V1 terminal
		hub.publish(format!(
			"\n======= RUNNING: {agent_name}\n     Agent path: {agent_path}",
		))
		.await;

		Ok(run_id)
	}

	pub async fn update_run_model_and_concurrency(
		&self,
		run_id: Id,
		model_name: &str,
		concurrency: usize,
	) -> Result<()> {
		let run_u = RunForUpdate {
			model: Some(model_name.to_string()),
			concurrency: Some(concurrency as i32),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		Ok(())
	}

	pub fn set_run_end_error(&self, run_id: Id, stage: Option<Stage>, err: &crate::Error) -> Result<()> {
		RunBmc::set_end_error(self.mm(), run_id, stage, err)?;
		Ok(())
	}

	/// Note: the rec log already happened (in the current design)
	/// This does not set the end time
	pub fn set_run_end_state_to_skip(&self, run_id: Id) -> Result<()> {
		RunBmc::update(
			self.mm(),
			run_id,
			RunForUpdate {
				end_state: Some(EndState::Skip),
				..Default::default()
			},
		)?;
		Ok(())
	}
}

/// Task Create/Update model
impl Runtime {
	pub async fn create_task(&self, run_id: Id, idx: usize, input: &Value) -> Result<Id> {
		let input_content = TypedContent::from_value(input);

		let task_c = TaskForCreate {
			run_id,
			idx: idx as i64,
			label: None,
			input_content: Some(input_content),
		};
		let id = TaskBmc::create(self.mm(), task_c)?;
		Ok(id)
	}

	pub async fn update_task_model_ov(&self, _run_id: Id, task_id: Id, model_name_ov: &str) -> Result<()> {
		let task_u = TaskForUpdate {
			model_ov: Some(model_name_ov.to_string()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		Ok(())
	}

	pub async fn update_task_usage(&self, _run_id: Id, task_id: Id, usage: &genai::chat::Usage) -> Result<()> {
		let task_u = TaskForUpdate::from_usage(usage);
		TaskBmc::update(self.mm(), task_id, task_u)?;
		Ok(())
	}

	pub async fn update_task_cost(&self, run_id: Id, task_id: Id, cost: f64) -> Result<()> {
		// -- Update Task
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

	/// NOTE: if the .content & .display is None, then, nothing is saved
	pub async fn update_task_output(&self, task_id: Id, output: &Value) -> Result<()> {
		let output_content = TypedContent::from_value(output);
		if output_content.content.is_none() && output_content.display.is_none() {
			return Ok(()); // Nothing to update
		}

		TaskBmc::update_output(self.mm(), task_id, output_content)?;
		Ok(())
	}

	pub fn set_task_end_error(&self, _run_id: Id, task_id: Id, stage: Option<Stage>, err: &crate::Error) -> Result<()> {
		TaskBmc::set_end_error(self.mm(), task_id, stage, err)?;
		Ok(())
	}

	/// Note: the rec log already happened (in the current design)
	/// This does not set the end time
	pub fn set_task_end_state_to_skip(&self, _run_id: Id, task_id: Id) -> Result<()> {
		TaskBmc::update(
			self.mm(),
			task_id,
			TaskForUpdate {
				end_state: Some(EndState::Skip),
				..Default::default()
			},
		)?;
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

	async fn rec_log_no_msg(
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

/// Rec for specialize event (Skip, FileSave,)
/// NOTE: Probably should put the end state as well
impl Runtime {
	pub async fn rec_skip_task(&self, run_id: Id, task_id: Id, stage: Stage, reason: Option<String>) -> Result<()> {
		let mm = self.mm();

		let reason_txt = reason.as_ref().map(|r| format!(" (Reason: {r})")).unwrap_or_default();

		// -- Update the Run end_skip_reason
		TaskBmc::update(
			mm,
			task_id,
			TaskForUpdate {
				end_skip_reason: reason.clone(),
				..Default::default()
			},
		)?;

		// -- Update the log
		let log_c = LogForCreate {
			run_id,
			task_id: Some(task_id),
			step: None,
			stage: Some(stage),
			message: reason,
			kind: Some(LogKind::AgentSkip),
		};
		LogBmc::create(mm, log_c)?;

		// -- publish for legacy
		get_hub()
			.publish(format!("Aipack Skip input at {stage} stage: {reason_txt}"))
			.await;

		Ok(())
	}

	/// NOTE: Probably shoul put the end state as well
	pub async fn rec_skip_run(&self, run_id: Id, stage: Stage, reason: Option<String>) -> Result<()> {
		let mm = self.mm();

		let reason_txt = reason.as_ref().map(|r| format!(" (Reason: {r})")).unwrap_or_default();

		// -- Update the Run end_skip_reason
		RunBmc::update(
			mm,
			run_id,
			RunForUpdate {
				end_skip_reason: reason.clone(),
				..Default::default()
			},
		)?;

		// -- Update the log
		let log_c = LogForCreate {
			run_id,
			task_id: None,
			step: None,
			stage: Some(stage),
			message: reason,
			kind: Some(LogKind::AgentSkip),
		};
		LogBmc::create(mm, log_c)?;

		// -- publish for legacy
		get_hub()
			.publish(format!("Aipack Skip input at {stage} stage: {reason_txt}"))
			.await;

		Ok(())
	}
}
