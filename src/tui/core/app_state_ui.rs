//! Convenient AppState getters and formatters for the View
//!
// region:    --- Imports
use crate::support::text::{self, format_duration_us, format_f64};
use crate::support::time::now_unix_time_us;
use crate::tui::AppState;
// endregion: --- Imports

/// Implement Run Related Data extractors
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
		if let Some(run) = self.current_run()
			&& let Some(cost) = run.total_cost
		{
			format!("${}", format_f64(cost))
		} else {
			"$...".to_string()
		}
	}

	pub fn current_run_concurrency_txt(&self) -> String {
		if let Some(run) = self.current_run()
			&& let Some(concurrency) = run.concurrency
		{
			concurrency.to_string()
		} else {
			"..".to_string()
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

		// Note: Will be 'no model' if no models
		let run_model = self.current_run_model_name();

		// Collect unique models while preserving encounter order.
		let mut uniques: Vec<String> = Vec::new();
		let mut some_no_ov = false;
		for task in tasks {
			if let Some(model) = &task.model_ov {
				if model != &run_model && !uniques.contains(model) {
					uniques.push(model.clone());
				}
			} else {
				some_no_ov = true;
			}
		}

		// NOTE: If tasks.len() == 0, uniques as well, so run_model (which is what we want. )
		match (uniques.len(), some_no_ov) {
			(0, _) => run_model,
			(n, true) => format!("{run_model} +{n}"),
			(1, false) => uniques.into_iter().next().unwrap_or_default(),
			(n, false) => format!("{} +{}", uniques.into_iter().next().unwrap_or_default(), n - 1),
		}
	}
}

/// Implement Task Related Data extractors
impl AppState {
	pub fn current_task_model_name(&self) -> String {
		self.current_task()
			.and_then(|r| r.model_ov.clone())
			.unwrap_or_else(|| self.current_run_model_name())
	}

	pub fn current_task_cost_txt(&self) -> String {
		if let Some(task) = self.current_task()
			&& let Some(cost) = task.cost
		{
			format!("${}", format_f64(cost))
		} else {
			"$...".to_string()
		}
	}

	/// Returns the cumulative duration of all tasks (formatted)  
	/// or `None` when there is **one task or fewer**.
	pub fn current_task_duration_txt(&self) -> String {
		if let Some(task) = self.current_task() {
			let duration = match (task.start, task.end) {
				(Some(start), Some(end)) => end.as_i64() - start.as_i64(),
				(Some(start), None) => now_unix_time_us() - start.as_i64(),
				_ => 0,
			};
			format_duration_us(duration)
		} else {
			"..".to_string()
		}
	}

	pub fn render_task_prompt_tokens_txt(&self) -> Option<String> {
		let task = self.current_task()?;
		let tk_prompt = task.tk_prompt_total?;

		let mut addl: Vec<String> = Vec::new();
		if let Some(tk_cached) = task.tk_prompt_cached {
			if tk_cached > 0 {
				addl.push(format!("{tk_cached} cached"));
			}
		}
		if let Some(tk_cache_creation) = task.tk_prompt_cache_creation {
			if tk_cache_creation > 0 {
				addl.push(format!("{tk_cache_creation} cache creation"));
			}
		}

		if !addl.is_empty() {
			Some(format!("{tk_prompt} ({})", addl.join(", ")))
		} else {
			Some(tk_prompt.to_string())
		}
	}

	pub fn render_task_completion_tokens_txt(&self) -> Option<String> {
		let task = self.current_task()?;
		let tk_completion = task.tk_completion_total?;

		if let Some(reasonning) = task.tk_completion_reasoning {
			Some(format!("{tk_completion} ({reasonning} reasoning)"))
		} else {
			Some(tk_completion.to_string())
		}
	}
}

/// Impl AppState formatters
impl AppState {
	pub fn memory_fmt(&self) -> String {
		let mem = self.memory();
		text::format_size_xfixed(mem)
	}

	pub fn cpu_fmt(&self) -> String {
		let cpu = self.cpu();
		text::format_f64(cpu)
	}
}
