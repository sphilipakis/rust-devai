use crate::store::rt_model::Run;
use crate::store::{Id, RunningState};

#[derive(Debug, Clone)]
pub struct RunItem {
	run: Run,
	indent: u32,
	ancestors: Vec<Id>,
}

// region:    --- RunItem Impl

/// Constructor
impl RunItem {
	pub fn new(run: Run, indent: u32, ancestors: Vec<Id>) -> Self {
		Self {
			run,
			indent,
			ancestors,
		}
	}
}

/// Getters
impl RunItem {
	pub fn id(&self) -> Id {
		self.run.id
	}

	pub fn run(&self) -> &Run {
		&self.run
	}

	#[allow(unused)]
	pub fn indent(&self) -> u32 {
		self.indent
	}
	pub fn parent_id(&self) -> Option<Id> {
		self.run.parent_id
	}

	#[allow(unused)]
	pub fn is_top_run(&self) -> bool {
		self.indent == 0
	}

	pub fn is_root(&self) -> bool {
		self.parent_id().is_none()
	}

	pub fn ancestors(&self) -> &[Id] {
		&self.ancestors
	}
}

// endregion: --- RunItem Impl

// region:    --- RunItem Froms

impl From<&RunItem> for RunningState {
	fn from(value: &RunItem) -> Self {
		value.run().into()
	}
}

// endregion: --- RunItem Froms
