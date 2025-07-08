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
			.constraints(vec![Constraint::Length(1), Constraint::Fill(1)])
			.areas(area);

		render_top(header_a, buf, state);
		render_logs(logs_a, buf, state);
	}
}

// region:    --- Render Helpers

fn render_top(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	// -- Prepare Header Data
	let agent_name = state.current_run_agent_name();
	let model_name = state.tasks_cummulative_models();
	let cost_txt = state.current_run_cost_txt();
	let duration_txt = state.current_run_duration_txt();

	let header_line = if let Some(cumul_txt) = state.tasks_cummulative_duration() {
		format!(
			"Agent: {agent_name}  Model: {model_name}  Cost: {cost_txt}  Duration: {duration_txt}  Cumulative: {cumul_txt}"
		)
	} else {
		format!("Agent: {agent_name}  Model: {model_name}  Cost: {cost_txt}  Duration: {duration_txt}")
	};

	Paragraph::new(header_line).render(area, buf);
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

