#![allow(unused)]

use crate::store::Id;
use crate::tui::core::Action;
use derive_more::From;
use ratatui::layout::{Position, Rect};

// region:    --- DataZones

#[derive(Debug, Default, Clone)]
pub struct LinkZones {
	/// The current line to which the new with the new link zone will be added from
	/// This allows to have the LinkZones::push_link_zone(relative_idx, ..)
	current_line: usize,

	/// The zones, keyed by their data reference.
	zones: Vec<LinkZone>,
}

impl LinkZones {
	pub fn set_current_line(&mut self, current_line: usize) {
		self.current_line = current_line;
	}

	pub fn push_link_zone(&mut self, rel_line_idx: usize, span_start: usize, span_end: usize, action: Action) {
		let line_idx = self.current_line + rel_line_idx;
		self.zones.push(LinkZone::new(line_idx, span_start, span_end, action));
	}

	pub fn into_zones(self) -> Vec<LinkZone> {
		self.zones
	}
}

// endregion: --- DataZones

// region:    --- DataZone

#[derive(Debug, Clone, Copy)]
pub struct LinkZone {
	pub line_idx: usize,
	pub span_start: usize,
	pub span_count: usize,
	pub action: Action,
}

/// Constructor
impl LinkZone {
	/// relative_line_idx is relative to the current_line
	pub fn new(line_idx: usize, span_start: usize, span_count: usize, action: Action) -> Self {
		Self {
			line_idx,
			span_start,
			span_count,
			action,
		}
	}
}

// endregion: --- DataZone

// region:    --- DataKey

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DataKey {
	Run { run_id: Id },
	Task { task_id: Id },
}

// endregion: --- DataKey
