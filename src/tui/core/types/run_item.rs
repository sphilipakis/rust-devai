use crate::store::rt_model::Run;
use crate::store::{Id, RunningState};
use std::collections::HashMap;

pub struct RunItem {
	run: Run,
	indent: u32,
}
/// This build the flat list of RunItem with the following rules
/// - First, the initial runs are order by most recent to oldest, which is what we want for the top order.
/// - The root item should have `indent = 0`
/// - Then, we order the remaining list from oldest to newest (ascending id())
/// - Then, include the children run items after the coresponding parent, with the appropriate ident
/// - And we return the flat list.
pub fn build_run_items(runs: Vec<Run>) -> Vec<RunItem> {
	// -- Early Exit
	if runs.is_empty() {
		return Vec::new();
	}

	// -- Build Roots & Children Map
	let mut children_map: HashMap<Id, Vec<Run>> = HashMap::new();
	let mut root_runs: Vec<Run> = Vec::new();

	for run in runs {
		if let Some(parent_id) = run.parent_id {
			children_map.entry(parent_id).or_default().push(run);
		} else {
			root_runs.push(run); // Keep original (most-recent-first) order.
		}
	}

	// -- Recursively Flatten
	fn push_with_children(out: &mut Vec<RunItem>, children_map: &mut HashMap<Id, Vec<Run>>, run: Run, indent: u32) {
		let id = run.id;
		out.push(RunItem::new(run, indent));

		if let Some(mut kids) = children_map.remove(&id) {
			// Oldest → Newest
			kids.sort_by_key(|r| r.id);
			for child in kids {
				push_with_children(out, children_map, child, indent + 1);
			}
		}
	}

	let mut flat: Vec<RunItem> = Vec::new();

	for run in root_runs {
		push_with_children(&mut flat, &mut children_map, run, 0);
	}

	// -- Orphan Handling (if any)
	if !children_map.is_empty() {
		let mut remaining: Vec<Run> = children_map.into_values().flatten().collect();
		remaining.sort_by_key(|r| r.id);
		for run in remaining {
			push_with_children(&mut flat, &mut HashMap::new(), run, 0);
		}
	}

	flat
}

// region:    --- RunItem Impl

/// Constructor
impl RunItem {
	pub fn new(run: Run, ident: u32) -> Self {
		Self { run, indent: ident }
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
}

// endregion: --- RunItem Impl

// region:    --- RunItem Froms

impl From<&RunItem> for RunningState {
	fn from(value: &RunItem) -> Self {
		value.run().into()
	}
}

// endregion: --- RunItem Froms
