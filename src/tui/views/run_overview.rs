use crate::store::Stage;
use crate::store::rt_model::{Log, LogBmc, LogKind, Run, Task};
use crate::tui::support::RectExt;
use crate::tui::views::support::{self, new_marker, ui_for_marker_section};
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
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
	support::extend_lines(&mut all_lines, ui_for_before_all(run, &logs, max_width, false), true);

	// -- Add the tasks ui
	support::extend_lines(&mut all_lines, ui_for_tasks(run, state.tasks(), max_width), true);

	// -- Add before all
	support::extend_lines(&mut all_lines, ui_for_after_all(run, &logs, max_width, false), true);

	// -- Add Error if present
	if let Some(err_id) = run.end_err_id {
		support::extend_lines(&mut all_lines, support::ui_for_err(state.mm(), err_id, max_width), true);
	}

	// -- Clamp scroll
	// TODO: Needs to have it's own scroll state.
	let line_count = all_lines.len();
	let max_scroll = line_count.saturating_sub(area.height as usize) as u16;
	if state.log_scroll() > max_scroll {
		state.set_log_scroll(max_scroll);
	}

	// -- Render All Content
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

fn ui_for_before_all(run: &Run, logs: &[Log], max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	ui_for_logs(run, logs, Some(Stage::BeforeAll), max_width, show_steps)
}

fn ui_for_after_all(run: &Run, logs: &[Log], max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	ui_for_logs(run, logs, Some(Stage::AfterAll), max_width, show_steps)
}

fn ui_for_logs(_run: &Run, logs: &[Log], stage: Option<Stage>, max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	let mut all_lines: Vec<Line> = Vec::new();

	let mut first_section = true;
	for log in logs {
		// Show or not step
		if !show_steps && matches!(log.kind, Some(LogKind::RunStep)) {
			continue;
		}

		if stage.is_some() && log.stage.is_none() {
			continue;
		}

		if let Some(stage) = stage
			&& let Some(log_stage) = log.stage
			&& stage != log_stage
		{
			continue;
		}

		if first_section {
			first_section = false
		} else {
			all_lines.push(Line::default()); // empty line (for now)
		}

		// Render log lines
		let log_lines = support::ui_for_log(log, max_width);
		all_lines.extend(log_lines);
	}

	all_lines
}

fn ui_for_tasks(_run: &Run, tasks: &[Task], _max_width: u16) -> Vec<Line<'static>> {
	if tasks.is_empty() {
		return Vec::new();
	}

	let mut spans_lines: Vec<Vec<Span<'static>>> = Vec::new();
	let tasks_len = tasks.len();

	for task in tasks {
		let mut task_line = task.ui_label(tasks_len);
		task_line.push(Span::raw("  "));
		task_line.extend(task.ui_stage_statuses_spans());
		spans_lines.push(task_line);
	}

	let marker = new_marker("Tasks:", styles::STL_SECTION_MARKER);

	ui_for_marker_section(vec![marker], vec![Span::raw(" ")], spans_lines)
}

// endregion: --- UI Builders
