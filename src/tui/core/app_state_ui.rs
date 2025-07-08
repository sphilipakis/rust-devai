//! Convenient AppState getters and formatters for the View
//!
// region:    --- Imports
use crate::support::text::{format_duration_us, format_float};
use crate::support::time::now_unix_time_us;
use crate::tui::AppState;
// endregion: --- Imports

impl AppState {
	pub fn current_run_duration_txt(&self) -> String {
		if let Some(run) = self.current_run() {
			let duration_us = match (run.start, run.end) {
				(Some(start), Some(end)) => end.as_i64() - start.as_i64(),
				(Some(start), None) => now_unix_time_us() - start.as_i64(),
				_ => 0,
			};
			format_duration_us(duration_us)
		} else {
			format_duration_us(0)
		}
	}

	pub fn current_run_cost_txt(&self) -> String {
		if let Some(run) = self.current_run() {
			if let Some(cost) = run.total_cost {
				format!("${}", format_float(cost))
			} else {
				"$...".to_string()
			}
		} else {
			"$...".to_string()
		}
	}

	/// Returns the agent name for the current run or `"no agent"` if none.
	pub fn current_run_agent_name(&self) -> String {
		self.current_run()
			.and_then(|r| r.agent_name.clone())
			.unwrap_or_else(|| "no agent".to_string())
	}

	/// Returns the model name for the current run or `"no model"` if none.
	pub fn current_run_model_name(&self) -> String {
		self.current_run()
			.and_then(|r| r.model.clone())
			.unwrap_or_else(|| "no model".to_string())
	}

	/// Returns the cumulative duration of all tasks (formatted)  
	/// or `None` when there is **one task or fewer**.
	pub fn tasks_cummulative_duration(&self) -> Option<String> {
		let tasks = self.tasks();
		if tasks.len() <= 1 {
			return None;
		}

		let mut cumul_us: i64 = 0;
		for task in tasks {
			let du = match (task.start, task.end) {
				(Some(start), Some(end)) => end.as_i64() - start.as_i64(),
				(Some(start), None) => now_unix_time_us() - start.as_i64(),
				_ => 0,
			};
			cumul_us += du;
		}

		Some(format_duration_us(cumul_us))
	}

	/// Returns a string describing the models used by the tasks of the
	/// current run.
	///
	/// Rules:
	/// - If all tasks share the same model, that model is returned.
	/// - If several different models are present, return the first two and
	///   append `+N` where `N` is the number of additional distinct models.
	/// - If no task has a model (or there are no tasks), fall back to the
	///   current run model (or `"no model"` when absent).
	pub fn tasks_cummulative_models(&self) -> String {
		let tasks = self.tasks();

		// Collect unique models while preserving encounter order.
		let mut uniques: Vec<String> = Vec::new();
		for task in tasks {
			if let Some(model) = &task.model {
				if !uniques.contains(model) {
					uniques.push(model.clone());
				}
			}
		}

		match uniques.len() {
			0 => self.current_run_model_name(),
			1 => uniques.first().cloned().unwrap(),
			2 => format!("{}, {}", uniques[0], uniques[1]),
			n => format!("{}, {} +{}", uniques[0], uniques[1], n - 2),
		}
	}
}
