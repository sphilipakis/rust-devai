use crate::tui::core::{AppState, MouseEvt};
use ratatui::layout::Rect;

/// Current Mouse Evt
impl AppState {
	pub fn mouse_evt(&self) -> Option<MouseEvt> {
		self.core.mouse_evt
	}

	pub fn last_mouse_evt(&self) -> Option<MouseEvt> {
		self.core.last_mouse_evt
	}

	/// Remove both the mouse_evt and last_mouse_evt
	/// This is good to avoid having a mouse event impacting the next redraw
	#[allow(unused)]
	pub fn clear_mouse_evts(&mut self) {
		self.core.mouse_evt = None;
		self.core.last_mouse_evt = None;
	}

	#[allow(unused)]
	pub fn is_mouse_over(&self, area: Rect) -> bool {
		self.core.mouse_evt.is_some_and(|m| area.contains(m.into()))
	}

	#[allow(unused)]
	pub fn is_mouse_down(&self) -> bool {
		self.core.mouse_evt.is_some_and(|m| m.is_down())
	}

	#[allow(unused)]
	pub fn is_mouse_up_only(&self) -> bool {
		self.core.mouse_evt.is_some_and(|m| m.is_up_only())
	}
}

/// Last Mouse Evt
impl AppState {
	pub fn is_last_mouse_over(&self, area: Rect) -> bool {
		self.core.last_mouse_evt.is_some_and(|m| area.contains(m.into()))
	}

	#[allow(unused)]
	pub fn is_last_mouse_down(&self) -> bool {
		self.core.last_mouse_evt.is_some_and(|m| m.is_down())
	}

	#[allow(unused)]
	pub fn is_last_mouse_up(&self) -> bool {
		self.core.last_mouse_evt.is_some_and(|m| m.is_up())
	}
}
