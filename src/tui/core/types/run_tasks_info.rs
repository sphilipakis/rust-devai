use crate::model::{EpochUs, Id};

#[derive(Debug, Clone)]
pub struct RunTasksInfo {
	run_id: Id,
	tasks_count: usize,
	last_task_mtime: Option<EpochUs>,
	tasks_cummulative_time_us: i64,
}

impl RunTasksInfo {
	pub fn new(
		run_id: Id,
		tasks_count: usize,
		last_task_mtime: Option<EpochUs>,
		tasks_cummulative_time_us: i64,
	) -> Self {
		Self {
			run_id,
			tasks_count,
			last_task_mtime,
			tasks_cummulative_time_us,
		}
	}

	pub fn run_id(&self) -> Id {
		self.run_id
	}

	pub fn tasks_count(&self) -> usize {
		self.tasks_count
	}

	pub fn last_task_mtime(&self) -> Option<EpochUs> {
		self.last_task_mtime
	}

	pub fn tasks_cummulative_time_us(&self) -> i64 {
		self.tasks_cummulative_time_us
	}
}
