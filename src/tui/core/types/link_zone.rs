#![allow(unused)]

use crate::store::Id;
use crate::tui::core::{Action, MouseEvt};
use crate::tui::support::UiExt as _;
use derive_more::From;
use ratatui::layout::{Position, Rect};
use ratatui::text::Span;

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

	pub fn inc_current_line_by(&mut self, amount: usize) {
		self.current_line += amount;
	}

	pub fn push_link_zone(&mut self, rel_line_idx: usize, span_start: usize, span_count: usize, action: Action) {
		let line_idx = self.current_line + rel_line_idx;
		self.zones.push(LinkZone::new(line_idx, span_start, span_count, action));
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

impl LinkZone {
	// check if it is mouse over the zone, and return the vec of over spans
	pub fn is_mouse_over<'a>(
		&self,
		//
		ref_area: Rect,
		scroll: u16,
		mouse_evt: Option<MouseEvt>,
		// the full line spans
		spans: &'a mut [Span<'static>],
	) -> Option<&'a mut [Span<'static>]> {
		let mouse_evt = mouse_evt?;
		let before_spans = spans.get(0..self.span_start)?;
		let before_width = before_spans.x_width();

		let zone_spans = spans.get_mut(self.span_start..self.span_start + self.span_count)?;

		let zone_area = Rect {
			x: ref_area.x + before_width,
			y: ref_area.y + self.line_idx as u16 - scroll, // not sure why +1
			width: zone_spans.x_width(),
			height: 1,
		};

		if mouse_evt.is_over(zone_area) {
			Some(zone_spans)
		} else {
			None
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
