use crate::model::Id;
use crate::model::Run;
use crate::tui::core::RunItem;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct RunItemStore {
	items: Vec<RunItem>,
	items_by_id: HashMap<Id, RunItem>,
}

impl RunItemStore {
	/// Returns the flat list of `RunItem`s.
	pub fn items(&self) -> &[RunItem] {
		&self.items
	}

	#[allow(unused)]
	pub fn get(&self, id: Id) -> Option<&RunItem> {
		self.items_by_id.get(&id)
	}

	/// Returns a list of direct children for a given `RunItem`.
	/// The children are ordered by their creation time (oldest first).
	#[allow(unused)]
	pub fn direct_children<'a>(&'a self, parent_item: &RunItem) -> Vec<&'a RunItem> {
		self.items
			.iter()
			.filter(|item| item.parent_id() == Some(parent_item.id()))
			.collect()
	}

	/// Returns all children (direct and indirect) for a given `RunItem`.
	pub fn all_children<'a>(&'a self, parent_item: &RunItem) -> Vec<&'a RunItem> {
		let children_ids: HashSet<Id> = parent_item.all_children_ids().iter().copied().collect();
		if children_ids.is_empty() {
			return Vec::new();
		}

		self.items.iter().filter(|item| children_ids.contains(&item.id())).collect()
	}
}

/// Contrustor
impl RunItemStore {
	pub fn new(runs: Vec<Run>) -> Self {
		// -- Early Exit
		if runs.is_empty() {
			return RunItemStore::default();
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
		fn push_with_children(
			out: &mut Vec<RunItem>,
			children_map: &mut HashMap<Id, Vec<Run>>,
			run: Run,
			indent: u32,
			ancestors: &[Id],
		) {
			let id = run.id;
			// This is the item for the current run
			out.push(RunItem::new(run, indent, ancestors.to_vec()));

			if let Some(mut kids) = children_map.remove(&id) {
				// Oldest → Newest
				kids.sort_by_key(|r| r.id);

				// The ancestors for all the direct children of this run.
				let mut child_ancestors = ancestors.to_vec();
				child_ancestors.push(id);

				for child in kids {
					push_with_children(out, children_map, child, indent + 1, &child_ancestors);
				}
			}
		}

		let mut flat: Vec<RunItem> = Vec::new();

		for run in root_runs {
			push_with_children(&mut flat, &mut children_map, run, 0, &[]);
		}

		// -- Orphan Handling (if any)
		if !children_map.is_empty() {
			let mut remaining: Vec<Run> = children_map.into_values().flatten().collect();
			remaining.sort_by_key(|r| r.id);
			for run in remaining {
				// Note: orphans will have an empty ancestor list (besides themselve)
				push_with_children(&mut flat, &mut HashMap::new(), run, 0, &[]);
			}
		}

		// -- Populate `all_children_ids` for each `RunItem`
		//    Iterate in reverse order (from children to parents) to build the `all_children_ids` map.
		let mut all_children_ids_by_id: HashMap<Id, Vec<Id>> = HashMap::new();

		// Create a map of items by parent_id for efficient lookup of direct children.
		let mut direct_children_by_parent_id: HashMap<Id, Vec<Id>> = HashMap::new();
		for item in &flat {
			if let Some(parent_id) = item.parent_id() {
				direct_children_by_parent_id.entry(parent_id).or_default().push(item.id());
			}
		}

		for item in flat.iter().rev() {
			let mut all_children_ids = Vec::new();

			// Look up direct children.
			if let Some(mut direct_children) = direct_children_by_parent_id.get(&item.id()).cloned() {
				direct_children.sort(); // Sort to be consistent with original logic
				for child_id in &direct_children {
					all_children_ids.push(*child_id);
					// Add grandchildren from the map we are building.
					if let Some(grand_children_ids) = all_children_ids_by_id.get(child_id) {
						all_children_ids.extend_from_slice(grand_children_ids);
					}
				}
			}
			all_children_ids_by_id.insert(item.id(), all_children_ids);
		}

		// Now, update the `all_children_ids` for each item in the `flat` vec.
		for item in &mut flat {
			if let Some(ids) = all_children_ids_by_id.get(&item.id()) {
				item.all_children_ids = ids.clone();
			}
		}

		let items_by_id = flat.iter().map(|item| (item.id(), item.clone())).collect();

		RunItemStore {
			items: flat,
			items_by_id,
		}
	}
}
