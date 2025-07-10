use crate::support::text::{format_duration_us, format_float};
use crate::tui::support::RectExt;
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};

pub struct SumView;

impl StatefulWidget for SumView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- Layout
		let [current_a, total_a] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![Constraint::Max(40), Constraint::Max(33)])
			.spacing(1)
			.areas(area);

		render_current(current_a, buf, state);
		render_total(total_a, buf, state);
	}
}

fn render_current(area: Rect, buf: &mut Buffer, state: &AppState) {
	Block::new().bg(styles::CLR_BKG_ACT).render(area, buf);

	let [status_a, metrics_a] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![Constraint::Min(28), Constraint::Fill(1)])
		.spacing(1)
		.areas(area);

	// -- Extract Status data
	let (agent_name, ended, total_cost) = if let Some(run) = state.current_run() {
		let agent_name = run.agent_name.as_deref().unwrap_or("no agent").to_string();
		let ended = run.end.is_some();
		let total_cost = run.total_cost;
		(agent_name, ended, total_cost)
	} else {
		("no agent".to_string(), false, None)
	};
	let duration = state.current_run_duration_txt();

	// -- Extract Tasks Data
	let mut done_count = 0;
	let total_tasks = state.tasks().len();
	for task in state.tasks().iter() {
		task.is_done().then(|| done_count += 1);
	}
	let running = total_tasks - done_count;
	let is_done = running == 0;

	// -- Render Left Side
	// Agent name
	// let run_id = state.current_run().map(|r| r.id.as_i64()).unwrap_or(-1);
	let status_a_inner = status_a.x_h_margin(1);

	let mut line_1 = Line::default();
	if ended {
		line_1.push_span(Span::styled("✔", styles::CLR_TXT_DONE));
	} else {
		line_1.push_span(Span::styled("▶", styles::CLR_TXT_RUNNING));
	};
	line_1.push_span(format!(" {agent_name}"));

	// Tasks
	let mut line_2 = Line::from(vec![
		Span::styled("  Tasks: ", styles::STL_TXT_ACT),
		Span::styled(format!("{done_count}"), styles::STL_TXT_ACT.green().bold()),
		Span::styled("/", styles::STL_TXT_ACT),
		Span::styled(format!("{total_tasks}"), styles::STL_TXT_ACT.bold()),
	]);
	if is_done {
		line_2.push_span(Span::styled(" (DONE)", styles::STL_TXT.dark_gray()));
	} else {
		line_2.push_span(Span::styled(
			format!(" ({running} running)"),
			styles::STL_TXT.dark_gray(),
		));
	}
	let text = Text::from(vec![line_1, line_2]);
	Paragraph::new(text).render(status_a_inner, buf);

	// -- Render Right Side
	let metrics_a_inner = metrics_a.x_h_margin(1);

	let line_1 = Line::from(vec![
		//
		Span::styled(duration, styles::STL_TXT),
	]);
	let mut line_2 = Line::default();
	if let Some(total_cost) = total_cost {
		let total_cost = format_float(total_cost);
		line_2.push_span(Span::styled(format!("~${total_cost}"), styles::STL_TXT));
	} else {
		line_2.push_span(Span::styled("~$...", styles::STL_TXT));
	}

	let text = Text::from(vec![line_1, line_2]);
	Paragraph::new(text).right_aligned().render(metrics_a_inner, buf);
}

fn render_total(area: Rect, buf: &mut Buffer, state: &AppState) {
	Block::new().bg(styles::CLR_BKG_GRAY_DARKER).render(area, buf);

	let content_a = area.x_h_margin(1);

	// -- Extract Data
	let mut duration_us: i64 = 0;
	let mut cost: f64 = 0.;
	let mut done_runs_count = 0;
	for run in state.runs() {
		run.is_done().then(|| done_runs_count += 1);
		if let Some(run_cost) = run.total_cost {
			cost += run_cost
		};
		if let (Some(start), Some(end)) = (run.start, run.end) {
			duration_us += end.as_i64() - start.as_i64();
		}
	}
	let running_run = state.runs().len() - done_runs_count;

	// -- Render status
	let line_1 = Line::from(vec![
		//
		Span::styled("Total Runs", styles::STL_TXT),
	]);
	let mut line_2 = Line::from(vec![
		Span::styled("Runs: ", styles::STL_TXT),
		Span::styled(format!("{done_runs_count}"), styles::STL_TXT.bold()),
	]);
	if running_run > 0 {
		line_2.push_span(Span::styled(format!(" ({running_run})"), styles::STL_TXT.dark_gray()))
	}
	let text = Text::from(vec![line_1, line_2]);
	Paragraph::new(text).render(content_a, buf);

	// -- Render Time
	let cost = format_float(cost);
	let line_1 = Line::from(vec![
		//
		Span::styled(format_duration_us(duration_us), styles::STL_TXT),
	]);
	let line_2 = Line::from(vec![
		//
		Span::styled(format!("${cost}"), styles::STL_TXT),
	]);
	let text = Text::from(vec![line_1, line_2]);
	Paragraph::new(text).right_aligned().render(content_a, buf);
}
