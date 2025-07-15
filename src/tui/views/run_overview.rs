use crate::store::Stage;
use crate::store::rt_model::{Log, LogBmc, LogKind, Run, Task};
use crate::tui::support::RectExt;
use crate::tui::views::support;
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget as _};

/// Placeholder view for *Before All* tab.
pub struct RunOverviewView;

impl StatefulWidget for RunOverviewView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let area = area.x_h_margin(1);

		// -- Render Body
		render_body(area, buf, state);
	}
}

fn render_body(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	let mut all_lines: Vec<Line> = Vec::new();

	let Some(run) = state.current_run() else {
		Paragraph::new("No current run").render(area, buf);
		return;
	};

	let logs = match LogBmc::list_for_run_only(state.mm(), run.id) {
		Ok(logs) => logs,
		Err(err) => {
			Paragraph::new(format!("Error fetch log for run. {err}")).render(area, buf);
			return;
		}
	};

	let max_width = area.width - 3; // for scroll

	// -- Add before all
	support::extend_lines(&mut all_lines, ui_for_before_all(run, logs, max_width, true), true);

	// -- Add the task ui
	support::extend_lines(&mut all_lines, ui_for_tasks(run, state.tasks(), max_width), true);

	// -- Clamp scroll
	// TODO: Needs to have it's own scroll state.
	let line_count = all_lines.len();
	let max_scroll = line_count.saturating_sub(area.height as usize) as u16;
	if state.log_scroll() > max_scroll {
		state.set_log_scroll(max_scroll);
	}

	// -- Render All Content
	// Block::new().bg(styles::CLR_BKG_PRIME).render(area, buf);
	let p = Paragraph::new(all_lines).scroll((state.log_scroll(), 0));
	p.render(area, buf);

	// -- Render Scrollbar
	let mut scrollbar_state = ScrollbarState::new(line_count).position(state.log_scroll() as usize);

	let scrollbar = Scrollbar::default()
		.orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
		.begin_symbol(Some("▲"))
		.end_symbol(Some("▼"));

	scrollbar.render(area, buf, &mut scrollbar_state);
}

// region:    --- UI Builders

fn ui_for_before_all(_run: &Run, logs: Vec<Log>, max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	let mut all_lines: Vec<Line> = Vec::new();

	for log in logs {
		if !matches!(log.stage, Some(Stage::BeforeAll)) {
			continue;
		}
		// Show or not step
		if !show_steps && matches!(log.kind, Some(LogKind::RunStep)) {
			continue;
		}

		// Render log lines
		let log_lines = support::ui_for_log(log, max_width);
		all_lines.extend(log_lines);
		all_lines.push(Line::default()); // empty line (for now)
	}

	all_lines
}

fn ui_for_tasks(_run: &Run, tasks: &[Task], max_width: u16) -> Vec<Line<'static>> {
	let mut content: Vec<String> = Vec::new();

	for task in tasks {
		let task_content = format!("Task {:?} - is ended: {}", task.idx, task.is_ended());
		content.push(task_content);
	}

	let content = content.join("\n\n");

	support::ui_for_marker_section(&content, ("Tasks", styles::STL_SECTION_MARKER), max_width, None)
}

// endregion: --- UI Builders

// region:    --- Support

// endregion: --- Support
