use crate::tui::core::{AppState, MouseEvt};
use ratatui::layout::Rect;

/// Mouse
impl AppState {
	// Getter
	pub fn mouse_evt(&self) -> Option<MouseEvt> {
		self.core.mouse_evt
	}

	pub fn is_mouse_over_area(&self, area: Rect) -> bool {
		let Some(last_mouse_evt) = self.core.last_mouse_evt else {
			return false;
		};
		area.contains(last_mouse_evt.into())
	}
}
