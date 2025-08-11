use crate::Result;
use crate::runtime::{RtLog, Runtime};
use crate::store::rt_model::{LogKind, RunBmc, RunForUpdate, TaskBmc, TaskForUpdate};
use crate::store::{EndState, Id, ModelManager, RunStep};
use crate::support::time::now_micro;
use derive_more::From;

#[derive(Debug, From)]
pub struct RtStep<'a> {
	runtime: &'a Runtime,
}

/// Constructor & Core Getters
impl<'a> RtStep<'a> {
	pub(super) fn new(runtime: &'a Runtime) -> Self {
		Self { runtime }
	}

	fn mm(&self) -> &ModelManager {
		self.runtime.mm()
	}

	fn rt_log(&self) -> RtLog<'_> {
		RtLog::new(self.runtime)
	}
}

/// Run Steps
impl<'a> RtStep<'a> {
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
		self.rt_log()
			.rec_log_no_msg(run_id, None, Some(RunStep::Start), None, Some(LogKind::RunStep))
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
		self.rt_log()
			.rec_log_no_msg(run_id, None, Some(RunStep::BaStart), None, Some(LogKind::RunStep))
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
		self.rt_log()
			.rec_log_no_msg(run_id, None, Some(RunStep::BaEnd), None, Some(LogKind::RunStep))
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
		self.rt_log()
			.rec_log_no_msg(run_id, None, Some(RunStep::TasksStart), None, Some(LogKind::RunStep))
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
		self.rt_log()
			.rec_log_no_msg(run_id, None, Some(RunStep::TasksEnd), None, Some(LogKind::RunStep))
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(
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
		self.rt_log()
			.rec_log_no_msg(run_id, None, Some(RunStep::AaStart), None, Some(LogKind::RunStep))
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
		self.rt_log()
			.rec_log_no_msg(run_id, None, Some(RunStep::AaEnd), None, Some(LogKind::RunStep))
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
		self.rt_log()
			.rec_log_no_msg(run_id, None, Some(RunStep::End), None, Some(LogKind::RunStep))
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
		self.rt_log()
			.rec_log_no_msg(run_id, None, Some(RunStep::End), None, Some(LogKind::RunStep))
			.await?;

		Ok(())
	}
}
