use crate::store::rt_model::{Log, LogBmc, LogKind};
use crate::tui::support::RectExt;
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
		// -- Layout Header |Logs
		let [header_a, _space_1, logs_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(2), // header
				Constraint::Max(1),    // space_1
				Constraint::Fill(1),   // logs
			])
			.areas(area);

		render_header(header_a, buf, state);

		// don't show the steps
		render_logs(logs_a, buf, state, false);
	}
}

fn render_header(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Prepare Data
	let model_name = state.current_task_model_name();
	let cost = state.current_task_cost_txt();
	let duration = state.current_task_duration_txt();
	let tk_prompt = state.render_task_prompt_tokens_txt();
	let tk_completion = state.render_task_completion_tokens_txt();

	let fist_call_width = 10;

	// -- Line 1 colums
	let [l1_label_1, l1_val_1, l1_label_2, l1_val_2, l1_label_3, l1_val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(fist_call_width), // Model
			Constraint::Length(20),              // Model value
			Constraint::Length(10),              // duration
			Constraint::Length(12),              // duration value
			Constraint::Length(7),               // duration
			Constraint::Length(20),              // duration value
		])
		.spacing(1)
		.areas(area.x_height(1));

	// -- Render Line 1
	// Model
	Paragraph::new("Model:")
		.style(styles::STL_TXT_LBL)
		.right_aligned()
		.render(l1_label_1, buf);
	Paragraph::new(model_name).style(styles::STL_TXT_VAL).render(l1_val_1, buf);
	// Duration
	Paragraph::new("Duration:")
		.style(styles::STL_TXT_LBL)
		.right_aligned()
		.render(l1_label_2, buf);
	Paragraph::new(duration).style(styles::STL_TXT_VAL).render(l1_val_2, buf);
	// Cost
	Paragraph::new("Cost:")
		.style(styles::STL_TXT_LBL)
		.right_aligned()
		.render(l1_label_3, buf);
	Paragraph::new(cost).style(styles::STL_TXT_VAL).render(l1_val_3, buf);

	// -- Line 2 Layout
	let [l2_label_1, l2_val_1] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(fist_call_width), // Tokens
			Constraint::Fill(1),
		])
		.spacing(1)
		.areas(area.x_move_top(1).x_height(1));

	// -- Line 2 render
	Paragraph::new("Tokens:")
		.style(styles::STL_TXT_LBL)
		.right_aligned()
		.render(l2_label_1, buf);
	let mut txt = String::new();
	if let Some(tk_prompt) = tk_prompt {
		txt.push_str(&format!("Prompt: {tk_prompt}"));
	}
	if let Some(tk_completion) = tk_completion {
		if !txt.is_empty() {
			txt.push_str("  ");
		}
		txt.push_str(&format!("Completion: {tk_completion}"));
	}
	if txt.is_empty() {
		if let Some(task) = state.current_task()
			&& task.is_done()
		{
			txt.push_str("No AI ran for this task.");
		} else {
			txt.push_str("...");
		}
	}
	Paragraph::new(txt).style(styles::STL_TXT_VAL).render(l2_val_1, buf);
}

fn render_logs(area: Rect, buf: &mut Buffer, state: &mut AppState, show_steps: bool) {
	// -- Fetch Logs
	let logs = if let Some(current_task) = state.current_task() {
		LogBmc::list_for_task(state.mm(), current_task.id)
	} else {
		Ok(Vec::new())
	};

	// -- Prepare content
	let content = match logs {
		Ok(logs) => {
			let max_width = area.width - 3; // for the scroll bar
			let mut lines: Vec<Line> = Vec::new();
			for log in logs {
				// Show or not step
				if !show_steps && matches!(log.kind, Some(LogKind::RunStep)) {
					continue;
				}

				// Render log lines
				let log_lines = render_log(log, max_width);
				lines.extend(log_lines);
				lines.push(Line::default()); // empty line (for now)
			}
			if lines.is_empty() {
				lines.push("No logs".into())
			}
			lines
		}
		Err(err) => vec![format!("LogBmc::list error. {err}").into()],
	};
	let line_count = content.len();

	// -- Clamp scroll
	let max_scroll = line_count.saturating_sub(area.height as usize) as u16;
	if state.log_scroll > max_scroll {
		state.log_scroll = max_scroll;
	}

	// -- Render content
	// Block::new().bg(styles::CLR_BKG_PRIME).render(area, buf);
	let p = Paragraph::new(content).scroll((state.log_scroll, 0));
	p.render(area, buf);

	// -- Render Scrollbar
	let mut scrollbar_state = ScrollbarState::new(line_count).position(state.log_scroll as usize);

	let scrollbar = Scrollbar::default()
		.orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
		.begin_symbol(Some("▲"))
		.end_symbol(Some("▼"));

	scrollbar.render(area, buf, &mut scrollbar_state);
}

// region:    --- Item Renderers

fn render_log(log: Log, max_width: u16) -> Vec<Line<'static>> {
	let Some(kind) = log.kind else {
		return vec![Line::raw(format!("Log [{}] has no kind", log.id))];
	};
	let content = match (log.message.as_ref(), log.kind.as_ref()) {
		(_, Some(LogKind::RunStep)) => log.step_as_str(),
		(Some(msg), _) => msg,
		(_, _) => "No Step not MSG for log",
	};

	let mark_txt = match kind {
		LogKind::RunStep => "Sys Step",
		LogKind::SysInfo => "Sys Info",
		LogKind::SysWarn => "Sys Warn",
		LogKind::SysError => "Sys Error",
		LogKind::SysDebug => "Sys Debug",
		LogKind::AgentPrint => "Agent Print",
	};

	render_section(content, mark_txt, 12, max_width)
}

// IN PROGRESS - need to refactor so that render_log uses it, so that we can use this function to render input and output
//               Should not have log.
fn render_section(content: &str, marker_txt: &str, marker_width: usize, max_width: u16) -> Vec<Line<'static>> {
	let spacer = " ";
	let width_spacer = spacer.len(); // won't work if no ASCII
	let width_content = (max_width as usize) - marker_width - width_spacer;

	// -- Mark Span
	let mark_span = Span::styled(format!("{marker_txt:>marker_width$}"), styles::STL_TXT_LBL);

	let msg_wrap = textwrap::wrap(content, width_content);
	let msg_wrap_len = msg_wrap.len();

	// -- First Content Line
	let mut msg_wrap_iter = msg_wrap.into_iter();
	let first_content = msg_wrap_iter.next().unwrap_or_default();
	let first_content_span = Span::raw(first_content.to_string());

	let first_line = Line::from(vec![
		//
		mark_span,
		Span::raw(" "), // must be equa
		first_content_span,
	]);

	// -- Lines
	let mut lines = vec![first_line];

	// -- Render other content line if present
	if msg_wrap_len > 2 {
		let left_spacing = " ".repeat(marker_width + width_spacer);
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

// endregion: --- Item Renderers
