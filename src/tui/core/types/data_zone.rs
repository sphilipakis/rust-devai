//! A TUI zone representing a data entity (type, id, and eventually property/ies)
#![allow(unused)]

use crate::store::Id;
use derive_more::From;
use ratatui::layout::{Position, Rect};

// region:    --- DataZones

#[derive(Debug, Default, Clone, From)]
pub struct DataZones {
	/// The zones, keyed by their data reference.
	zones: Vec<DataZone>,
}

impl DataZones {
	pub fn find_data_key(&self, pos: Position, x_offset: u16, y_offset: u16) -> Option<DataKey> {
		let new_pos = Position::new(pos.x - x_offset, pos.y - y_offset);

		for zone in &self.zones {
			// tracing::debug!(
			// 	"mouse pos: {pos} - new pos: {new_pos:?} - zone area: {:?}",
			// 	zone.area
			// );
			if zone.area.contains(new_pos) {
				return Some(zone.data_key);
			}
		}
		None
	}
}

// endregion: --- DataZones

// region:    --- DataZone

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DataZone {
	data_key: DataKey,
	area: Rect,
}

/// Constructor
impl DataZone {
	#[allow(unused)]
	pub fn new_for_run(area: Rect, run_id: Id) -> Self {
		Self {
			data_key: DataKey::Run { run_id },
			area,
		}
	}

	pub fn new_for_task(area: Rect, task_id: Id) -> Self {
		Self {
			data_key: DataKey::Task { task_id },
			area,
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
