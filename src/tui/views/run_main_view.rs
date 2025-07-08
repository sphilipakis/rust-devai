use crate::store::rt_model::LogBmc;
use crate::tui::AppState;
use crate::tui::styles::CLR_BKG_GRAY_DARKER;
use crate::tui::support::RectExt;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget as _};

pub struct RunMainView {}

impl StatefulWidget for RunMainView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		Block::new().bg(CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Layout Header | Logs
		let [header_a, logs_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![Constraint::Length(2), Constraint::Fill(1)])
			.areas(area);

		render_top(header_a, buf, state);
		render_logs(logs_a, buf, state);
	}
}

// region:    --- Render Helpers

fn render_top(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Prepare Data
	let agent_name = state.current_run_agent_name();
	let model_name = state.tasks_cummulative_models();
	let cost_txt = state.current_run_cost_txt();
	let duration_txt = state.current_run_duration_txt();

	// Tasks progress and optional cumulative duration.
	let total_tasks = state.tasks().len();
	let done_tasks = state.tasks().iter().filter(|t| t.is_done()).count();
	let mut tasks_txt = format!("{done_tasks}/{total_tasks}");
	if let Some(cumul_txt) = state.tasks_cummulative_duration() {
		tasks_txt = format!("{tasks_txt} ({cumul_txt})");
	}

	// -- Layout Helpers
	// 6 columns: label / value repeated 3 times.
	let cols = vec![
		Constraint::Length(10), // "Agent:" / "Model:" labels
		Constraint::Length(20), // Values for Agent / Model
		Constraint::Length(10), // "Duration:" / "Cost:"
		Constraint::Length(8),  // Values for Duration / Cost
		Constraint::Length(7),  // "Tasks:" label
		Constraint::Length(10), // Tasks value or blank
	];

	let [line_1_a, line_2_a] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![Constraint::Length(1), Constraint::Length(1)])
		.areas(area);

	// -- Render Line 1
	let [l1_label_1, l1_val_1, l1_label_2, l1_val_2, l1_label_3, l1_val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(cols.clone())
		.spacing(1)
		.areas(line_1_a);

	Paragraph::new("Agent:").right_aligned().render(l1_label_1, buf);
	Paragraph::new(agent_name).render(l1_val_1, buf);

	Paragraph::new("Duration:").right_aligned().render(l1_label_2, buf);
	Paragraph::new(duration_txt).render(l1_val_2, buf);

	Paragraph::new("Tasks:").right_aligned().render(l1_label_3, buf);
	Paragraph::new(tasks_txt).render(l1_val_3, buf);

	// -- Render Line 2
	let [l2_label_1, l2_val_1, l2_label_2, l2_val_2, _l2_label_3, _l2_val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(cols)
		.spacing(1)
		.areas(line_2_a);

	Paragraph::new("Model:").right_aligned().render(l2_label_1, buf);
	Paragraph::new(model_name).render(l2_val_1, buf);

	Paragraph::new("Cost:").right_aligned().render(l2_label_2, buf);
	Paragraph::new(cost_txt).render(l2_val_2, buf);
}

fn render_logs(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Fetch Logs
	let logs = if let Some(current_run) = state.current_run() {
		LogBmc::list_for_display(state.mm(), current_run.id)
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
						"{:<2} - {:<10} - {:<8} - {:<12} - {}",
						log.id,
						log.level.map(|v| v.to_string()).unwrap_or_else(|| "no-level".to_string()),
						log.stage.map(|v| v.to_string()).unwrap_or_else(|| "no-stage".to_string()),
						log.step.map(|v| v.to_string()).unwrap_or_else(|| "no-step".to_string()),
						log.message.map(|v| v.to_string()).unwrap_or_else(|| "no-message".to_string())
					)
				})
				.collect();
			if lines.is_empty() {
				"No logs for this run.".to_string()
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
