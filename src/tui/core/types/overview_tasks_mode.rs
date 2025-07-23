use derive_more::Display;

#[derive(Debug, Clone, Copy, Display, Eq, PartialEq)]
pub enum OverviewTasksMode {
	Auto,
	List,
	Grid,
}

impl OverviewTasksMode {
	pub fn next(self) -> Self {
		match self {
			OverviewTasksMode::Auto => OverviewTasksMode::List,
			OverviewTasksMode::List => OverviewTasksMode::Grid,
			OverviewTasksMode::Grid => OverviewTasksMode::Auto,
		}
	}
}
