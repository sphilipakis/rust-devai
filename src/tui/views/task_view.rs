use crate::store::ModelManager;
use crate::store::rt_model::{Log, LogBmc, LogKind, Task, TaskBmc};
use crate::tui::support::RectExt;
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget as _};

/// Renders the content of a task. For now, the logs.
pub struct TaskView;

const MARKER_WIDTH: usize = 10;

impl StatefulWidget for TaskView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// let show_model_row = state.tasks().len() > 1;
		let show_model_row = true; // for now we always show

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
		render_sections(logs_a, buf, state, false);
	}
}

fn render_header(area: Rect, buf: &mut Buffer, state: &mut AppState, show_model_row: bool) {
	// -- Prepare Data
	let model_name = state.current_task_model_name();
	let cost = state.current_task_cost_txt();
	let duration = state.current_task_duration_txt();
	let prompt_tk = state.render_task_prompt_tokens_fmt();
	let completion_tk = state.render_task_completion_tokens_fmt();

	let first_call_width = 10;

	// -- Columns layout
	let [label_1, val_1, label_2, val_2, label_3, val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(first_call_width), // Model / Prompt
			Constraint::Length(22),               //
			Constraint::Length(12),               // Cost / Completion
			Constraint::Length(9),                //
			Constraint::Length(13),               // Duration
			Constraint::Fill(1),                  //
		])
		.spacing(1)
		.areas(area);

	let mut current_row = 0;

	// -- Render Model Row
	if show_model_row {
		current_row += 1;

		Paragraph::new("Model:")
			.style(styles::STL_FIELD_LBL)
			.right_aligned()
			.render(label_1.x_row(current_row), buf);
		// NOTE: here a little chack to have maximum space for model name
		Paragraph::new(model_name)
			.style(styles::STL_FIELD_VAL)
			.render(val_1.x_row(current_row).x_width(26), buf);

		// NOTE: Here we use Span to give a little bit more space to Model
		Paragraph::new(Span::styled("  Cost:", styles::STL_FIELD_LBL))
			.right_aligned()
			.render(label_2.x_row(current_row), buf);
		Paragraph::new(cost)
			.style(styles::STL_FIELD_VAL)
			.render(val_2.x_row(current_row), buf);

		Paragraph::new("Duration:")
			.style(styles::STL_FIELD_LBL)
			.right_aligned()
			.render(label_3.x_row(current_row), buf);
		Paragraph::new(duration)
			.style(styles::STL_FIELD_VAL)
			.render(val_3.x_row(current_row), buf);
	}

	// -- Render Row for tokens
	current_row += 1;
	Paragraph::new("Prompt:")
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(label_1.x_row(current_row), buf);
	Paragraph::new(prompt_tk)
		.style(styles::STL_FIELD_VAL)
		.render(val_1.x_row(current_row), buf);

	Paragraph::new("Completion:")
		.style(styles::STL_FIELD_LBL)
		.right_aligned()
		.render(label_2.x_row(current_row), buf);
	Paragraph::new(completion_tk)
		.style(styles::STL_FIELD_VAL)
		.render(val_2.union(val_3).x_row(current_row), buf);
}

fn render_sections(area: Rect, buf: &mut Buffer, state: &mut AppState, show_steps: bool) {
	// -- Get the current task (return early)
	let Some(task) = state.current_task() else {
		Line::raw("No Current Task").render(area, buf);
		return;
	};

	// -- Setup UI Lines
	let mut all_lines: Vec<Line> = Vec::new();
	let max_width = area.width - 3; // for scroll

	// -- Add Input
	all_lines.extend(ui_for_input(state.mm(), task, max_width));
	all_lines.push(Line::default());

	// -- Add Logs Lines
	all_lines.extend(ui_for_logs(state.mm(), task, max_width, show_steps));

	// -- Add output if end
	if task.output_uid.is_some() {
		all_lines.extend(ui_for_output(state.mm(), task, max_width));
		all_lines.push(Line::default());
	}

	// -- Clamp scroll
	let line_count = all_lines.len();
	let max_scroll = line_count.saturating_sub(area.height as usize) as u16;
	if state.log_scroll() > max_scroll {
		state.set_log_scroll(max_scroll);
	}

	// -- Render content
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

// region:    --- Ui Helpers

fn ui_for_input(mm: &ModelManager, task: &Task, max_width: u16) -> Vec<Line<'static>> {
	let marker_txt = "Input:";
	let marker_style = styles::STL_SECTION_MARKER_INPUT;
	match TaskBmc::get_input_for_display(mm, task) {
		Ok(Some(content)) => ui_for_section(&content, (marker_txt, marker_style), max_width),
		Ok(None) => ui_for_section("no input found", (marker_txt, marker_style), max_width),
		Err(err) => ui_for_section(
			&format!("Error getting input. {err}"),
			(marker_txt, marker_style),
			max_width,
		),
	}
}

fn ui_for_output(mm: &ModelManager, task: &Task, max_width: u16) -> Vec<Line<'static>> {
	let marker_txt = "Output:";
	let marker_style = styles::STL_SECTION_MARKER_OUTPUT;
	match TaskBmc::get_output_for_display(mm, task) {
		Ok(Some(content)) => ui_for_section(&content, (marker_txt, marker_style), max_width),
		Ok(None) => ui_for_section("no output found", (marker_txt, marker_style), max_width),
		Err(err) => ui_for_section(
			&format!("Error getting output. {err}"),
			(marker_txt, marker_style),
			max_width,
		),
	}
}

fn ui_for_logs(mm: &ModelManager, task: &Task, max_width: u16, show_steps: bool) -> Vec<Line<'static>> {
	// -- Fetch Logs
	let logs = LogBmc::list_for_task(mm, task.id);

	let mut lines: Vec<Line> = Vec::new();

	// -- Prepare content
	match logs {
		Ok(logs) => {
			for log in logs {
				// Show or not step
				if !show_steps && matches!(log.kind, Some(LogKind::RunStep)) {
					continue;
				}

				// Render log lines
				let log_lines = ui_for_log(log, max_width);
				lines.extend(log_lines);
				lines.push(Line::default()); // empty line (for now)
			}
			if lines.is_empty() {
				// lines.push("No logs".into())
			}
		}
		Err(err) => lines.push(format!("LogBmc::list error. {err}").into()),
	};

	lines
}

fn ui_for_log(log: Log, max_width: u16) -> Vec<Line<'static>> {
	let Some(kind) = log.kind else {
		return vec![Line::raw(format!("Log [{}] has no kind", log.id))];
	};
	let content = match (log.message.as_ref(), log.kind.as_ref()) {
		(_, Some(LogKind::RunStep)) => log.step_as_str(),
		(Some(msg), _) => msg,
		(_, _) => "No Step not MSG for log",
	};

	let marker_txt_style = match kind {
		LogKind::RunStep => ("Sys Step", styles::STL_SECTION_MARKER),
		LogKind::SysInfo => ("Sys Info", styles::STL_SECTION_MARKER),
		LogKind::SysWarn => ("Sys Warn", styles::STL_SECTION_MARKER),
		LogKind::SysError => ("Sys Error", styles::STL_SECTION_MARKER),
		LogKind::SysDebug => ("Sys Debug", styles::STL_SECTION_MARKER),
		LogKind::AgentPrint => ("Print:", styles::STL_SECTION_MARKER),
	};

	ui_for_section(content, marker_txt_style, max_width)
}

/// This is the task view record section with the marker and content, for each log line, or for input, output, (pins in the future)
/// NOTE: Probably can make Line lifetime same as content (to avoid string duplication). But since needs to be indented, probably not a big win.
fn ui_for_section(content: &str, (marker_txt, marker_style): (&str, Style), max_width: u16) -> Vec<Line<'static>> {
	let spacer = " ";
	let width_spacer = spacer.len(); // won't work if no ASCII
	let width_content = (max_width as usize) - MARKER_WIDTH - width_spacer;

	// -- Mark Span
	let mark_span = Span::styled(format!("{marker_txt:>MARKER_WIDTH$}"), marker_style);

	tracing::debug!("Content for section:\n{content}");
	let msg_wrap = textwrap::wrap(content, width_content);
	tracing::debug!("Wrapped for section:\n{msg_wrap:?}");

	let msg_wrap_len = msg_wrap.len();

	// -- First Content Line
	let mut msg_wrap_iter = msg_wrap.into_iter();
	let first_content = msg_wrap_iter.next().unwrap_or_default();
	let first_content_span = Span::raw(first_content.to_string());

	let first_line = Line::from(vec![
		//
		mark_span,
		Span::raw(" "),
		first_content_span,
	]);

	// -- Lines
	let mut lines = vec![first_line];

	// -- Render other content line if present
	if msg_wrap_len > 1 {
		let left_spacing = " ".repeat(MARKER_WIDTH + width_spacer);
		for line_content in msg_wrap_iter {
			let line = Line::raw(format!("{left_spacing}{line_content}"));
			lines.push(line)
		}
	}

	// -- Return lines
	lines
}

#[allow(unused)]
fn first_line_truncate(s: &str, max: usize) -> String {
	s.lines().next().unwrap_or("").chars().take(max).collect()
}

// endregion: --- Ui Helpers
