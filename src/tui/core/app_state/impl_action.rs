use crate::tui::core::{Action, AppState};

impl AppState {
	pub fn action(&self) -> Option<&Action> {
		self.core.do_action.as_ref()
	}

	pub fn set_action(&mut self, action: impl Into<Action>) {
		let action = action.into();
		self.core.do_action = Some(action);
	}

	pub fn clear_action(&mut self) {
		self.core.do_action = None;
	}
}
