use crate::store::ModelManager;
use crate::store::rt_model::{Log, LogBmc, LogKind, Run, Task, TaskBmc};
use crate::tui::support::RectExt;
use crate::tui::views::support;
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget as _};

/// Renders the content of a task. For now, the logs.
pub struct TaskView;

impl StatefulWidget for TaskView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let show_model_row = state.tasks().len() > 1;
		// let show_model_row = true;

		// -- Layout Header | Logs
		let header_height = if show_model_row { 2 } else { 1 };
		let [header_a, _space_1, logs_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(header_height), // header
				Constraint::Max(1),                // space_1
				Constraint::Fill(1),               // logs
			])
			.areas(area);

		render_header(header_a, buf, state, show_model_row);

		// don't show the steps
		render_body(logs_a, buf, state, false);
	}
}

fn render_header(area: Rect, buf: &mut Buffer, state: &mut AppState, show_model_row: bool) {
	// -- Prepare Data
	let model_name = state.current_task_model_name();
	let cost = state.current_task_cost_fmt();
	let duration = state.current_task_duration_txt();
	let prompt_tk = state.current_task_prompt_tokens_fmt();
	let completion_tk = state.current_task_completion_tokens_fmt();

	let first_call_width = 10;

	// -- Columns layout
	let [label_1, val_1, label_2, val_2, label_3, val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(first_call_width), // Model / Prompt
			Constraint::Length(25),               //
			Constraint::Length(9),                // Cost / Completion
			Constraint::Length(9),                //
			Constraint::Length(13),               // Duration
			Constraint::Fill(1),                  //
		])
		.spacing(1)
		.areas(area);

	let mut current_row = 0;

	// When color debug:
	let stl_field_val = if state.debug_clr() != 0 {
		styles::STL_FIELD_VAL.fg(ratatui::style::Color::Indexed(state.debug_clr()))
	} else {
		styles::STL_FIELD_VAL
	};

	// -- Render Model Row
	if show_model_row {
		current_row += 1;

		Paragraph::new("Model:")
			.style(styles::STL_FIELD_LBL)
			.right_aligned()
			.render(label_1.x_row(current_row), buf);
		// NOTE: here a little chack to have maximum space for model name
		Paragraph::new(model_name)
			.style(stl_field_val)
			.render(val_1.x_row(current_row).x_width(26), buf);

		// NOTE: Here we use Span to give a little bit more space to Model
		Paragraph::new(Span::styled("  Cost:", styles::STL_FIELD_LBL))
			.right_aligned()
			.render(label_2.x_row(current_row), buf);
		Paragraph::new(cost).style(stl_field_val).render(val_2.x_row(current_row), buf);

		Paragraph::new("Duration:")
			.style(styles::STL_FIELD_LBL)
			.right_aligned()
			.render(label_3.x_row(current_row), buf);
		Paragraph::new(duration)
			.style(stl_field_val)
			.render(val_3.x_row(current_row), buf);
	}

	// -- Render Row for tokens
	current_row += 1;
	Paragraph::new("Prompt:")
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(label_1.x_row(current_row), buf);
	Paragraph::new(prompt_tk)
		.style(stl_field_val)
		.render(val_1.x_row(current_row), buf);

	Paragraph::new("Compl:")
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(label_2.x_row(current_row), buf);
	Paragraph::new(completion_tk)
		.style(stl_field_val)
		.render(val_2.union(val_3).x_row(current_row), buf);
}

fn render_body(area: Rect, buf: &mut Buffer, state: &mut AppState, show_steps: bool) {
	// -- Get the current task (return early)
	let Some(run) = state.current_run() else {
		Line::raw("No Current Run").render(area, buf);
		return;
	};
	let Some(task) = state.current_task() else {
		Line::raw("No Current Task").render(area, buf);
		return;
	};
	// -- Fetch Logs
	let logs = match LogBmc::list_for_task(state.mm(), task.id) {
		Ok(logs) => logs,
		Err(err) => {
			Paragraph::new(format!("LogBmc::list error. {err}")).render(area, buf);
			return;
		}
	};

	// -- Setup UI Lines
	let mut all_lines: Vec<Line> = Vec::new();
	let max_width = area.width - 3; // for scroll

	// -- Add Input
	support::extend_lines(&mut all_lines, ui_for_input(state.mm(), task, max_width), true);

	// -- Add Before AI Logs Lines
	support::extend_lines(
		&mut all_lines,
		ui_for_before_ai_logs(task, &logs, max_width, show_steps),
		false,
	);

	// -- Add AI Lines
	support::extend_lines(&mut all_lines, ui_for_ai(run, task, max_width), true);

	// -- Add After AI Logs Lines
	support::extend_lines(
		&mut all_lines,
		ui_for_after_ai_logs(task, &logs, max_width, show_steps),
		false,
	);

	// -- Add output if end
	if task.output_uid.is_some() {
		support::extend_lines(&mut all_lines, ui_for_output(state.mm(), task, max_width), true);
	}

	// -- Add Error if present
	if let Some(err_id) = task.end_err_id {
		support::extend_lines(&mut all_lines, support::ui_for_err(state.mm(), err_id, max_width), true);
	}

	// -- Clamp scroll
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

fn ui_for_input(mm: &ModelManager, task: &Task, max_width: u16) -> Vec<Line<'static>> {
	let marker_txt = "Input:";
	let marker_style = styles::STL_SECTION_MARKER_INPUT;
	match TaskBmc::get_input_for_display(mm, task) {
		Ok(Some(content)) => support::ui_for_marker_section(&content, (marker_txt, marker_style), max_width, None),
		Ok(None) => Vec::new(),
		Err(err) => support::ui_for_marker_section(
			&format!("Error getting input. {err}"),
			(marker_txt, marker_style),
			max_width,
			None,
		),
	}
}

fn ui_for_ai(run: &Run, task: &Task, max_width: u16) -> Vec<Line<'static>> {
	let marker_txt = "AI:";
	let marker_style_active = styles::STL_SECTION_MARKER_AI;
	let marker_stype_inactive = styles::STL_SECTION_MARKER;
	let model_name = task
		.model_ov
		.as_ref()
		.or(run.model.as_ref())
		.map(|v| v.as_str())
		.unwrap_or_default();

	let ai_stage_done = task.ai_start.is_some() && task.ai_end.is_some();

	let (content, style) = match (ai_stage_done, task.ai_gen_start, task.ai_gen_end) {
		(_, Some(_start), None) => (
			Some(format!("➜ Sending prompt to AI model {model_name}.")),
			marker_style_active,
		),
		(_, Some(_start), Some(_end)) => {
			// let cost = state.current_task_cost_fmt();
			// let compl = state.current_task_completion_tokens_fmt();
			(Some(format!("✔ AI model {model_name} responded.")), marker_style_active)
		}
		(true, None, None) => (
			Some(". No instruction given. GenAI Skipped.".to_string()),
			marker_stype_inactive,
		),
		_ => (None, marker_stype_inactive),
	};

	if let Some(content) = content {
		support::ui_for_marker_section(&content, (marker_txt, style), max_width, None)
	} else {
		Vec::new()
	}
}

fn ui_for_output(mm: &ModelManager, task: &Task, max_width: u16) -> Vec<Line<'static>> {
	let marker_txt = "Output:";
	let marker_style = styles::STL_SECTION_MARKER_OUTPUT;
	match TaskBmc::get_output_for_display(mm, task) {
		Ok(Some(content)) => support::ui_for_marker_section(&content, (marker_txt, marker_style), max_width, None),
		Ok(None) => support::ui_for_marker_section("no output found", (marker_txt, marker_style), max_width, None),
		Err(err) => support::ui_for_marker_section(
			&format!("Error getting output. {err}"),
			(marker_txt, marker_style),
			max_width,
			None,
		),
	}
}

fn ui_for_before_ai_logs(task: &Task, logs: &[Log], max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	let ai_start: i64 = task.ai_start.map(|v| v.as_i64()).unwrap_or(i64::MAX);

	let logs = logs.iter().filter(|v| v.ctime.as_i64() < ai_start);

	ui_for_logs(logs, max_width, show_steps)
}

fn ui_for_after_ai_logs(task: &Task, logs: &[Log], max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	let ai_start: i64 = task.ai_start.map(|v| v.as_i64()).unwrap_or(i64::MAX);

	let logs = logs.iter().filter(|v| v.ctime.as_i64() > ai_start);

	ui_for_logs(logs, max_width, show_steps)
}

fn ui_for_logs<'a>(logs: impl IntoIterator<Item = &'a Log>, max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	let mut lines: Vec<Line> = Vec::new();
	for log in logs {
		// Show or not step
		if !show_steps && matches!(log.kind, Some(LogKind::RunStep)) {
			continue;
		}

		// Render log lines
		let log_lines = support::ui_for_log(log, max_width);
		lines.extend(log_lines);
		lines.push(Line::default()); // empty line (for now)
	}

	lines
}

#[allow(unused)]
fn first_line_truncate(s: &str, max: usize) -> String {
	s.lines().next().unwrap_or("").chars().take(max).collect()
}

// endregion: --- UI Builders
