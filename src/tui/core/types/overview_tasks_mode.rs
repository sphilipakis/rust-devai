use derive_more::Display;

const TASKS_GRID_THRESHOLD: usize = 10;

#[derive(Debug, Clone, Copy, Display, Eq, PartialEq)]
pub enum OverviewTasksMode {
	Auto,
	List,
	Grid,
}

impl OverviewTasksMode {
	pub fn next(self, tasks_len: usize) -> Self {
		let auto_is_grid = tasks_len >= TASKS_GRID_THRESHOLD;

		match (self, auto_is_grid) {
			(OverviewTasksMode::Auto, true) => OverviewTasksMode::List,
			(OverviewTasksMode::Auto, false) => OverviewTasksMode::Grid,
			(OverviewTasksMode::List, true) => OverviewTasksMode::Auto,
			(OverviewTasksMode::List, false) => OverviewTasksMode::Grid,
			(OverviewTasksMode::Grid, true) => OverviewTasksMode::List,
			(OverviewTasksMode::Grid, false) => OverviewTasksMode::Auto,
		}
	}

	pub fn is_grid(&self, tasks_len: usize) -> bool {
		match self {
			OverviewTasksMode::Auto => tasks_len >= TASKS_GRID_THRESHOLD,
			OverviewTasksMode::List => false,
			OverviewTasksMode::Grid => true,
		}
	}
}
