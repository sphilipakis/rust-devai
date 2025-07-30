use crate::tui::core::AppState;

impl AppState {
	/// Returns the agent name for the current run or `"no agent"` if none.
	pub fn current_run_has_prompt_parts(&self) -> Option<bool> {
		self.current_run_item().and_then(|r| r.run().has_prompt_parts)
	}

	/// Returns the agent name for the current run or `"no agent"` if none.
	pub fn current_run_has_task_stages(&self) -> Option<bool> {
		self.current_run_item().and_then(|r| r.run().has_task_stages)
	}

	/// Return true if there is a end skip reason
	pub fn current_run_has_skip(&self) -> bool {
		self.current_run_item()
			.map(|r| r.run().end_skip_reason.is_some())
			.unwrap_or_default()
	}
}
