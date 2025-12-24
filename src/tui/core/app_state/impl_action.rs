use crate::tui::core::{AppState, UiAction};

impl AppState {
	pub fn action(&self) -> Option<&UiAction> {
		self.core.do_action.as_ref()
	}

	pub fn set_action(&mut self, action: impl Into<UiAction>) {
		let action = action.into();
		self.core.do_action = Some(action);
		self.trigger_redraw();
	}

	pub fn clear_action(&mut self) {
		self.core.do_action = None;
	}
}
