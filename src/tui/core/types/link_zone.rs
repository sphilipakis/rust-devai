#![allow(unused)]

use crate::model::Id;
use crate::tui::core::{MouseEvt, UiAction};
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

	/// Optional grouping for section-wide hover selection.
	next_group_id: u32,
}

impl LinkZones {
	pub fn set_current_line(&mut self, current_line: usize) {
		self.current_line = current_line;
	}

	pub fn inc_current_line_by(&mut self, amount: usize) {
		self.current_line += amount;
	}

	pub fn push_link_zone(&mut self, rel_line_idx: usize, span_start: usize, span_count: usize, action: UiAction) {
		let line_idx = self.current_line + rel_line_idx;
		self.zones.push(LinkZone::new(line_idx, span_start, span_count, action));
	}

	/// Start a new group and return its id. Zones pushed with this id will be treated as a section.
	pub fn start_group(&mut self) -> u32 {
		let id = self.next_group_id;
		self.next_group_id = self.next_group_id.wrapping_add(1);
		id
	}

	/// Push a zone that belongs to a group (section-wide hover/click).
	pub fn push_group_zone(
		&mut self,
		rel_line_idx: usize,
		span_start: usize,
		span_count: usize,
		group_id: u32,
		action: UiAction,
	) {
		let line_idx = self.current_line + rel_line_idx;
		self.zones.push(LinkZone::new_with_group(
			line_idx,
			span_start,
			span_count,
			Some(group_id),
			action,
		));
	}

	pub fn into_zones(self) -> Vec<LinkZone> {
		self.zones
	}
}

// endregion: --- DataZones

// region:    --- DataZone

#[derive(Debug, Clone)]
pub struct LinkZone {
	pub line_idx: usize,
	pub span_start: usize,
	pub span_count: usize,
	pub action: UiAction,
	pub group_id: Option<u32>,
}

/// Constructor
impl LinkZone {
	/// relative_line_idx is relative to the current_line
	pub fn new(line_idx: usize, span_start: usize, span_count: usize, action: UiAction) -> Self {
		Self {
			line_idx,
			span_start,
			span_count,
			action,
			group_id: None,
		}
	}

	pub fn new_with_group(
		line_idx: usize,
		span_start: usize,
		span_count: usize,
		group_id: Option<u32>,
		action: UiAction,
	) -> Self {
		Self {
			line_idx,
			span_start,
			span_count,
			action,
			group_id,
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

		// Ensure the zone line is within the visible body area rows.
		let line_idx = self.line_idx;
		let scroll_usize = scroll as usize;
		let visible_top = scroll_usize;
		let visible_bottom = scroll_usize + ref_area.height as usize;
		if line_idx < visible_top || line_idx >= visible_bottom {
			return None;
		}

		let before_spans = spans.get(0..self.span_start)?;
		let before_width = before_spans.x_width();

		let zone_spans = spans.get_mut(self.span_start..self.span_start + self.span_count)?;

		let visible_row = (line_idx - scroll_usize) as u16;
		let zone_area = Rect {
			x: ref_area.x + before_width,
			y: ref_area.y + visible_row,
			width: zone_spans.x_width(),
			height: 1,
		};

		if mouse_evt.is_over(zone_area) {
			Some(zone_spans)
		} else {
			None
		}
	}

	/// Return a mutable slice for this zone span range on a given line's spans.
	pub fn spans_slice_mut<'a>(&self, spans: &'a mut [Span<'static>]) -> Option<&'a mut [Span<'static>]> {
		spans.get_mut(self.span_start..self.span_start + self.span_count)
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
