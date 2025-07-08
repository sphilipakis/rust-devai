use crate::store::rt_model::LogBmc;
use crate::tui::AppState;
use crate::tui::styles::CLR_BKG_GRAY_DARKER;
use crate::tui::support::RectExt;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget as _};

pub struct RunContentView {}

impl StatefulWidget for RunContentView {
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

	// -- Prepare Items
	let mut items: Vec<ListItem> = vec![];
	match logs {
		Ok(logs) => {
			for log in logs {
				let entry = if let Some(msg) = log.message {
					format!("{} - {} - {msg}", log.run_id, log.id)
				} else {
					let msg = log
						.step
						.map(|s| s.to_string())
						.unwrap_or_else(|| format!("No msg or step for log id {}", log.id));
					format!("{} - {} - {msg}", log.run_id, log.id)
				};
				items.push(ListItem::new(entry));
			}
		}
		Err(err) => items.push(ListItem::new(format!("LogBmc::list error. {err}"))),
	}

	let list_w = List::new(items);
	let mut list_s = ListState::default();
	StatefulWidget::render(list_w, area.x_margin(1), buf, &mut list_s);
}

// endregion: --- Render Helpers
