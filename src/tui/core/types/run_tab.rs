#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RunTab {
	Overview,
	Tasks,
}

impl RunTab {
	pub fn next(self) -> Self {
		match self {
			RunTab::Overview => RunTab::Tasks,
			RunTab::Tasks => RunTab::Tasks,
		}
	}

	pub fn prev(self) -> Self {
		match self {
			RunTab::Overview => RunTab::Overview,
			RunTab::Tasks => RunTab::Overview,
		}
	}
}
