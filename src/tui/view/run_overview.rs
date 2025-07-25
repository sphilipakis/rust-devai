use crate::store::rt_model::{Log, LogBmc, LogKind, Task};
use crate::store::{EndState, RunningState, Stage};
use crate::tui::AppState;
use crate::tui::core::{Action, LinkZones, ScrollIden};
use crate::tui::support::UiExt as _;
use crate::tui::view::support::{self, RectExt as _};
use crate::tui::view::{comp, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget as _};

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

	// -- Init the scroll area
	state.set_scroll_area(SCROLL_IDEN, area);

	// -- Determine tasks mode
	let tasks_len = state.tasks().len();

	let is_grid = state.overview_tasks_mode().is_grid(tasks_len);

	// -- Prep
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

	let mut link_zones = LinkZones::default();
	let mut all_lines: Vec<Line> = Vec::new();

	// -- Add before all
	support::extend_lines(&mut all_lines, ui_for_before_all(&logs, max_width, false), true);

	link_zones.set_current_line(all_lines.len());

	// -- Add the tasks ui
	let task_list_lines = if is_grid {
		ui_for_task_grid(state.tasks(), max_width, &mut link_zones)
	} else {
		ui_for_task_list(state.tasks(), max_width, &mut link_zones)
	};
	support::extend_lines(&mut all_lines, task_list_lines, true);

	link_zones.set_current_line(all_lines.len());

	// -- Add after all
	support::extend_lines(&mut all_lines, ui_for_after_all(&logs, max_width, false), true);

	link_zones.set_current_line(all_lines.len());

	// -- Add Error if present
	if let Some(err_id) = state.current_run().and_then(|r| r.end_err_id) {
		support::extend_lines(&mut all_lines, comp::ui_for_err(state.mm(), err_id, max_width), true);
	}

	link_zones.set_current_line(all_lines.len());

	// -- Clamp scroll
	// TODO: Needs to have it's own scroll state.
	let line_count = all_lines.len();
	let scroll = state.clamp_scroll(SCROLL_IDEN, line_count);

	// -- Perform the Click on a link zone
	for zone in link_zones.into_zones() {
		if let Some(line) = all_lines.get_mut(zone.line_idx) {
			if let Some(hover_spans) = zone.is_mouse_over(area, scroll, state.last_mouse_evt(), &mut line.spans) {
				for span in hover_spans {
					span.style.fg = Some(style::CLR_TXT_BLUE);
					if is_grid {
						span.style.bg = Some(style::CLR_BKG_BLACK);
					}
					span.style = span.style.add_modifier(Modifier::BOLD);
				}
				if state.is_mouse_up() {
					state.set_action(zone.action);
					state.trigger_redraw();
					// Note: Little trick to not show hover on the next tasks tab screen
					state.clear_mouse_evts();
				}
			}
		}
	}

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

fn ui_for_task_list(tasks: &[Task], max_width: u16, link_zones: &mut LinkZones) -> Vec<Line<'static>> {
	if tasks.is_empty() {
		return Vec::new();
	}

	// -- Prep
	let tasks_len = tasks.len();

	// let mut line: u16 = 0;
	let (marker, marker_spacer) = tasks_marker();
	let marker_width = marker.x_width();
	let marker_spacer_width = marker_spacer.x_width();
	let _marker_and_spacer_width = marker_width + marker_spacer_width;
	let marker_prefix_spans_len = marker.len() + marker_spacer.len();

	let content_width = max_width.saturating_sub(marker_spacer_width + marker_width);
	let gap_span = Span::raw("  ");
	let gap_width = gap_span.width() as u16;

	let mut all_lines: Vec<Vec<Span<'static>>> = Vec::new();

	// -- render legend (on top of list)
	// build legend_line
	all_lines.push(ui_for_legend(tasks));
	all_lines.push(Vec::new());

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
	for (idx, task) in tasks.iter().enumerate() {
		let mut task_line = task.ui_label(None, label_a.width, tasks_len);
		let task_id = task.id;

		// -- Link Zone
		// +2 for the space + ico (from the ui_label), 2 to take space and label text
		// NOTE: This should probably be part of the task facade (should not make those assumption here)
		link_zones.push_link_zone(idx, marker_prefix_spans_len + 2, 2, Action::GoToTask { task_id });

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

fn ui_for_task_grid(tasks: &[Task], max_width: u16, link_zones: &mut LinkZones) -> Vec<Line<'static>> {
	if tasks.is_empty() {
		return Vec::new();
	}

	// -- Prep
	let tasks_len = tasks.len();

	// let mut line: u16 = 0;
	let (marker, marker_spacer) = tasks_marker();
	let marker_width = marker.x_width();
	let marker_spacer_width = marker_spacer.x_width();
	let marker_prefix_spans_len = marker.len() + marker_spacer.len();

	let content_width = max_width.saturating_sub(marker_spacer_width + marker_width);
	let gap_span = Span::raw(" ");
	let gap_width = gap_span.width() as u16;

	let mut all_lines: Vec<Vec<Span<'static>>> = Vec::new();

	// -- render legend (on top of list)
	// build legend_line
	all_lines.push(ui_for_legend(tasks));
	all_lines.push(Vec::new());

	// -- Render tasks
	let mut line: Vec<Span<'static>> = Vec::new();
	let max_num = tasks_len;
	for task in tasks {
		let task_block = task.ui_short_block(max_num);

		let zone_span_start = line.len();
		let zone_span_count = task_block.len();

		// -- decide the create new line
		if line.x_width() + task_block.x_width() + gap_width <= content_width {
			// We append
			line.extend(task_block);
			// line.push(gap_span.clone());
		}
		// otherwise create a new line
		else {
			// end the previous line
			all_lines.push(line);
			// new lines
			// all_lines.push(vec![Span::raw("")]);
			// start the new one
			line = task_block;
			// line.push(gap_span.clone());
		}

		// -- Link Zone
		// +2 for the space + ico (fomrom the ui_label), 2 to take space and label text
		// NOTE: This should probably be part of the task facade (should not make those assumption here)
		let task_id = task.id;
		link_zones.push_link_zone(
			all_lines.len(),
			marker_prefix_spans_len + zone_span_start,
			zone_span_count,
			Action::GoToTask { task_id },
		);
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

fn ui_for_legend(tasks: &[Task]) -> Vec<Span<'static>> {
	let mut count_done = 0;
	let mut count_waiting = 0;
	let mut count_skip = 0;
	let mut count_err = 0;
	let mut count_ai = 0;

	// -- Process counts
	for task in tasks {
		match RunningState::from(task) {
			RunningState::NotScheduled => (), // this is for other state, ignore for now.
			RunningState::Waiting => count_waiting += 1,
			RunningState::Running => {
				if task.is_ai_running() {
					count_ai += 1;
				}
			}
			RunningState::Ended(end_state) => match end_state {
				Some(EndState::Ok) => count_done += 1,
				Some(EndState::Err) => count_err += 1,
				Some(EndState::Skip) => count_skip += 1,
				Some(EndState::Cancel) => (), // TODO: handle cancel
				None => (),
			},
		}
	}

	// -- Build the UI
	let num_width = 4;
	let mut legend_line = vec![
		// The Done
		Span::styled("Done:", style::CLR_BKG_RUNNING_DONE),
		Span::raw(format!(" {count_done:<num_width$} ")),
	];
	if count_ai > 0 {
		legend_line.push(Span::styled("AI:", style::CLR_BKG_RUNNING_AI));
		legend_line.push(Span::raw(format!(" {count_ai:<num_width$} ")));
	}
	if count_skip > 0 {
		legend_line.push(Span::styled("Skip:", style::CLR_BKG_RUNNING_SKIP));
		legend_line.push(Span::raw(format!(" {count_skip:<num_width$} ")));
	}
	if count_waiting > 0 {
		legend_line.push(Span::styled("Queue:", style::CLR_TXT_650));
		legend_line.push(Span::raw(format!(" {count_waiting:<num_width$} ")));
	}
	if count_err > 0 {
		legend_line.push(Span::styled("Error:", style::CLR_BKG_RUNNING_ERR));
		legend_line.push(Span::raw(format!(" {count_err:<num_width$} ")));
	}

	legend_line
}

// endregion: --- Support

// region:    --- UI Event Processing

// NOTE: Probably need a area_offset
// #[allow(unused)]
// fn process_mouse_for_task_list(state: &mut AppState, task_list_zones: LinkZones, x_offset: u16, y_offset: u16) {
// 	if let Some(mouse_evt) = state.mouse_evt()
// 		&& mouse_evt.is_click()
// 	{
// 		let data_ref = task_list_zones.find_data_key(mouse_evt.position(), x_offset, y_offset);
// 		// NOTE: now select the right data_ref
// 		if let Some(_data_ref) = data_ref {
// 			// tracing::debug!("data_ref: {data_ref:?}");
// 		}
// 		// TODO: ...
// 	}
// }

// endregion: --- UI Event Processing
