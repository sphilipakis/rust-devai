use crate::store::Stage;
use crate::store::rt_model::{Log, LogBmc, LogKind, Task};
use crate::tui::AppState;
use crate::tui::core::{DataZones, ScrollIden};
use crate::tui::view::support::RectExt as _;
use crate::tui::view::support::{self, UiExt as _};
use crate::tui::view::{comp, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget as _};

const TASKS_GRID_THRESHOLD: usize = 10;

/// Placeholder view for *Before All* tab.
pub struct RunOverviewView;

/// Component scroll identifiers
impl RunOverviewView {
	const BODY_SCROLL_IDEN: ScrollIden = ScrollIden::OverviewContent;

	const SCROLL_IDENS: &[&ScrollIden] = &[&Self::BODY_SCROLL_IDEN];

	pub fn clear_scroll_idens(state: &mut AppState) {
		state.clear_scroll_zone_areas(Self::SCROLL_IDENS);
	}
}

impl StatefulWidget for RunOverviewView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let area = area.x_h_margin(1);

		// -- Render Body
		render_body(area, buf, state);
	}
}

fn render_body(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	const SCROLL_IDEN: ScrollIden = RunOverviewView::BODY_SCROLL_IDEN;
	// -- Int the scroll area
	state.set_scroll_area(SCROLL_IDEN, area);

	// -- Prep
	let mut all_lines: Vec<Line> = Vec::new();

	let Some(run_id) = state.current_run().map(|r| r.id) else {
		Paragraph::new("No current run").render(area, buf);
		return;
	};

	let logs = match LogBmc::list_for_run_only(state.mm(), run_id) {
		Ok(logs) => logs,
		Err(err) => {
			Paragraph::new(format!("Error fetch log for run. {err}")).render(area, buf);
			return;
		}
	};

	let max_width = area.width - 3; // for scroll

	// -- Add before all
	support::extend_lines(&mut all_lines, ui_for_before_all(&logs, max_width, false), true);

	// -- Add the tasks ui
	//let tasks_list_start_y = all_lines.len() as u16;
	let tasks_len = state.tasks().len();
	let task_list_lines = if tasks_len < TASKS_GRID_THRESHOLD {
		ui_for_task_list(state.tasks(), max_width)
	} else {
		ui_for_task_grid(state.tasks(), max_width)
	};
	support::extend_lines(&mut all_lines, task_list_lines, true);

	// -- TO UPDATE - WIP - PRocess the datazone click
	// process_mouse_for_task_list(state, task_list_dzones, area.x, area.y + tasks_list_start_y);

	// -- Add before all
	support::extend_lines(&mut all_lines, ui_for_after_all(&logs, max_width, false), true);

	// -- Add Error if present
	if let Some(err_id) = state.current_run().and_then(|r| r.end_err_id) {
		support::extend_lines(&mut all_lines, comp::ui_for_err(state.mm(), err_id, max_width), true);
	}

	// -- Clamp scroll
	// TODO: Needs to have it's own scroll state.
	let line_count = all_lines.len();
	let scroll = state.clamp_scroll(SCROLL_IDEN, line_count);

	// -- Render All Content
	let p = Paragraph::new(all_lines).scroll((scroll, 0));
	p.render(area, buf);

	// -- Render Scrollbar
	let content_size = line_count.saturating_sub(area.height as usize);
	let mut scrollbar_state = ScrollbarState::new(content_size).position(scroll as usize);

	let scrollbar = Scrollbar::default()
		.orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
		.begin_symbol(Some("▲"))
		.end_symbol(Some("▼"));
	scrollbar.render(area, buf, &mut scrollbar_state);
}

// region:    --- UI Builders

fn ui_for_before_all(logs: &[Log], max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	ui_for_logs(logs, Some(Stage::BeforeAll), max_width, show_steps)
}

fn ui_for_after_all(logs: &[Log], max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	ui_for_logs(logs, Some(Stage::AfterAll), max_width, show_steps)
}

fn ui_for_logs(logs: &[Log], stage: Option<Stage>, max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
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
		let log_lines = comp::ui_for_log(log, max_width);
		all_lines.extend(log_lines);
	}

	all_lines
}

fn ui_for_task_list(tasks: &[Task], max_width: u16) -> Vec<Line<'static>> {
	if tasks.is_empty() {
		return Vec::new();
	}

	// -- Prep
	let tasks_len = tasks.len();

	// let mut line: u16 = 0;
	let (marker, marker_spacer) = tasks_marker();
	let marker_width = marker.x_width();
	let marker_spacer_width = marker_spacer.x_width();

	let content_width = max_width.saturating_sub(marker_spacer_width + marker_width);
	let gap_span = Span::raw("  ");
	let gap_width = gap_span.width() as u16;

	// -- Layout
	let [label_a, _, input_a, _, _ai_a, _, output_a] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(8),         // label_a
			Constraint::Length(gap_width), // gap
			Constraint::Fill(3),           // input_a
			Constraint::Length(gap_width), // gap
			Constraint::Length(6),         // ai_a (hardcode in task.ui_ai())
			Constraint::Length(gap_width), // gap
			Constraint::Fill(5),           // output_a
		])
		.areas(Rect::new(0, 0, content_width, 1));

	// --  Build the UI lines
	let mut all_lines: Vec<Vec<Span<'static>>> = Vec::new();
	for task in tasks {
		let mut task_line = task.ui_label(label_a.width, tasks_len);

		// -- Make the data zone
		// let x = marker_width + marker_spacer_width;
		// let data_task_area = Rect {
		// 	x,
		// 	y: line,
		// 	width: task_line.x_total_width(),
		// 	height: 1,
		// };
		// let data_zone = DataZone::new_for_task(data_task_area, task.id);
		// data_zones.push(data_zone);

		// -- Gap
		task_line.push(gap_span.clone());

		// -- Add Input
		let input_spans = task.ui_input(input_a.width);
		task_line.extend(input_spans);

		// -- Gap
		task_line.push(gap_span.clone());

		// -- Add AI
		let ai_spans = task.ui_ai();
		task_line.extend(ai_spans);

		// -- Gap
		task_line.push(gap_span.clone());

		// -- Add Output or skip
		if task.has_skip() {
			let skip_spans = task.ui_skip(output_a.width);
			task_line.extend(skip_spans);
		} else {
			let output_spans = task.ui_output(output_a.width);
			task_line.extend(output_spans);
		}

		// -- Add Sum iteams
		// task_line.extend(task.ui_sum_spans());

		all_lines.push(task_line);

		// line += 1;
	}

	// -- Build the marker component
	comp::ui_for_marker_section(marker, marker_spacer, all_lines)
}

fn ui_for_task_grid(tasks: &[Task], max_width: u16) -> Vec<Line<'static>> {
	if tasks.is_empty() {
		return Vec::new();
	}

	// -- Prep
	let tasks_len = tasks.len();

	// let mut line: u16 = 0;
	let (marker, marker_spacer) = tasks_marker();
	let marker_width = marker.x_width();
	let marker_spacer_width = marker_spacer.x_width();

	let content_width = max_width.saturating_sub(marker_spacer_width + marker_width);
	let gap_span = Span::raw(" ");
	let gap_width = gap_span.width() as u16;

	// -- Render
	let mut all_lines: Vec<Vec<Span<'static>>> = Vec::new();
	let mut line: Vec<Span<'static>> = Vec::new();
	let max_num = tasks_len;
	for task in tasks {
		let task_block = task.ui_short_block(max_num);
		// -- decide the create new lin
		if line.x_width() + task_block.x_width() + gap_width <= content_width {
			// We append
			line.push(gap_span.clone());
			line.extend(task_block);
		}
		// otherwise create a new line
		else {
			// end the previous line
			all_lines.push(line);
			// start the new one
			line = task_block;
		}
	}

	// -- add the last line
	all_lines.push(line);

	// -- Build the marker component
	comp::ui_for_marker_section(marker, marker_spacer, all_lines)
}

// endregion: --- UI Builders

// region:    --- Support

fn tasks_marker() -> (Vec<Span<'static>>, Vec<Span<'static>>) {
	let marker = vec![comp::new_marker("Tasks:", style::STL_SECTION_MARKER)];
	let marker_spacer = vec![Span::raw(" ")];
	(marker, marker_spacer)
}

// endregion: --- Support

// region:    --- UI Event Processing

// NOTE: Probably need a area_offset
#[allow(unused)]
fn process_mouse_for_task_list(state: &mut AppState, task_list_zones: DataZones, x_offset: u16, y_offset: u16) {
	if let Some(mouse_evt) = state.mouse_evt()
		&& mouse_evt.is_click()
	{
		let data_ref = task_list_zones.find_data_key(mouse_evt.position(), x_offset, y_offset);
		// NOTE: now select the right data_ref
		if let Some(_data_ref) = data_ref {
			// tracing::debug!("data_ref: {data_ref:?}");
		}
		// TODO: ...
	}
}

// endregion: --- UI Event Processing
