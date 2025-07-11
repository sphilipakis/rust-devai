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

		render_logs(logs_a, buf, state);
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
			let max_width = area.width - 3; // for the scroll bar
			let mut lines: Vec<Line> = Vec::new();
			for log in logs {
				// let txt = format!(
				// 	"{:<3} - {:<4} - {:<10} - {:<8} - {:<15} - {}",
				// 	log.id,
				// 	log.task_id.map(|v| v.to_string()).unwrap_or_default(),
				// 	log.kind.map(|v| v.to_string()).unwrap_or_else(|| "no-level".to_string()),
				// 	log.stage.map(|v| v.to_string()).unwrap_or_else(|| "no-stage".to_string()),
				// 	log.step.map(|v| v.to_string()).unwrap_or_else(|| "no-step".to_string()),
				// 	log.message.map(|v| v.to_string()).unwrap_or_else(|| "no-message".to_string())
				// );
				// lines.push(txt.into());
				let log_lines = render_log(log, max_width);
				lines.extend(log_lines);
				lines.push(Line::default()); // empty line
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

// region:    --- Log Renderers

fn render_log(log: Log, max_width: u16) -> Vec<Line<'static>> {
	let Some(kind) = log.kind else {
		return vec![Line::raw(format!("Log [{}] has no kind", log.id))];
	};

	let width_marker = 12;
	let spacer = " ";
	let width_spacer = spacer.len(); // won't work if no ASCII
	let width_content = (max_width as usize) - width_marker - width_spacer;

	// -- Mark Span
	// TODO: need to return style as well
	let mark_txt = match kind {
		LogKind::RunStep => "Sys Step",
		LogKind::SysInfo => "Sys Info",
		LogKind::SysWarn => "Sys Warn",
		LogKind::SysError => "Sys Error",
		LogKind::SysDebug => "Sys Debug",
		LogKind::AgentPrint => "Agent Print",
	};
	let mark_span = Span::styled(format!("{mark_txt:>width_marker$}"), styles::STL_TXT_LBL);

	let msg_wrap = log.message.as_ref().map(|msg| textwrap::wrap(msg, width_content));

	// -- First Content Line
	let first_content = match (msg_wrap.as_ref(), kind) {
		(_, LogKind::RunStep) => log.step_as_str().to_string(),
		(Some(msg_wrap), _) => msg_wrap.first().map(|s| s.to_string()).unwrap_or_default(),
		(_, _) => format!("No Step not MSG for log {}", log.id),
	};
	let first_content_span = Span::raw(first_content);

	let first_line = Line::from(vec![
		//
		mark_span,
		Span::raw(" "), // must be equa
		first_content_span,
	]);

	// -- Lines
	let mut lines = vec![first_line];

	// -- Render other content line if present
	if let Some(msg_wrap) = msg_wrap
		&& msg_wrap.len() > 1
	{
		let mut msg_wrap_iter = msg_wrap.into_iter();
		// we skip the first line, already printed
		msg_wrap_iter.next();
		// NOTE: for now we need to close this left_spacing because of the return type
		//       With might be able to return a type to avoid new string
		let left_spacing = " ".repeat(width_marker + width_spacer);
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

// endregion: --- Log Renderers
