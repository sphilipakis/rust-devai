use crate::store::rt_model::LogBmc;
use crate::tui::AppState;
use crate::tui::support::RectExt;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget as _};

/// Renders the content of a task. For now, the logs.
pub struct TaskView;

impl StatefulWidget for TaskView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		render_logs(area, buf, state);
	}
}

// region:    --- Render Helpers

fn render_logs(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Fetch Logs
	let logs = if let Some(current_task) = state.current_task() {
		LogBmc::list_for_task(state.mm(), current_task.id)
	} else {
		Ok(Vec::new())
	};

	// -- Prepare content
	let content = match logs {
		Ok(logs) => {
			let lines: Vec<String> = logs
				.into_iter()
				.map(|log| {
					format!(
						"{:<3} - {:<4} - {:<10} - {:<8} - {:<15} - {}",
						log.id,
						log.task_id.map(|v| v.to_string()).unwrap_or_default(),
						log.kind.map(|v| v.to_string()).unwrap_or_else(|| "no-level".to_string()),
						log.stage.map(|v| v.to_string()).unwrap_or_else(|| "no-stage".to_string()),
						log.step.map(|v| v.to_string()).unwrap_or_else(|| "no-step".to_string()),
						log.message.map(|v| v.to_string()).unwrap_or_else(|| "no-message".to_string())
					)
				})
				.collect();
			if lines.is_empty() {
				"No logs for this task yet...".to_string()
			} else {
				lines.join("\n")
			}
		}
		Err(err) => format!("LogBmc::list error. {err}"),
	};
	let line_count = content.lines().count();
	let area_with_margin = area.x_margin(1);

	// -- Clamp scroll
	let max_scroll = line_count.saturating_sub(area_with_margin.height as usize) as u16;
	if state.log_scroll > max_scroll {
		state.log_scroll = max_scroll;
	}

	// -- Render content
	let p = Paragraph::new(content).scroll((state.log_scroll, 0));
	p.render(area_with_margin, buf);

	// -- Render Scrollbar
	let mut scrollbar_state = ScrollbarState::new(line_count).position(state.log_scroll as usize);

	let scrollbar = Scrollbar::default()
		.orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
		.begin_symbol(Some("▲"))
		.end_symbol(Some("▼"));

	scrollbar.render(area, buf, &mut scrollbar_state);
}

// endregion: --- Render Helpers
