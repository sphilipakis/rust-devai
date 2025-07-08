use crate::store::rt_model::LogBmc;
use crate::support::text::{format_duration_us, format_float};
use crate::support::time::now_unix_time_us;
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

		// -- Prepare Header Data
		let (model_name, cost_txt, duration_txt) = if let Some(run) = state.current_run() {
			let model_name = run.agent_name.as_deref().unwrap_or("no model").to_string();

			let cost_txt = if let Some(cost) = run.total_cost {
				format!("${}", format_float(cost))
			} else {
				"$...".to_string()
			};

			let duration_us = match (run.start, run.end) {
				(Some(start), Some(end)) => end.as_i64() - start.as_i64(),
				(Some(start), None) => now_unix_time_us() - start.as_i64(),
				_ => 0,
			};

			(model_name, cost_txt, format_duration_us(duration_us))
		} else {
			("no model".to_string(), "$...".to_string(), format_duration_us(0))
		};

		// -- Cumulative Duration
		let mut cumul_us: i64 = 0;
		for run in state.tasks() {
			let du = match (run.start, run.end) {
				(Some(start), Some(end)) => end.as_i64() - start.as_i64(),
				(Some(start), None) => now_unix_time_us() - start.as_i64(),
				_ => 0,
			};
			cumul_us += du;
		}
		let cumul_txt = format_duration_us(cumul_us);

		// -- Render Header
		let header_line =
			format!("Model: {model_name}  Cost: {cost_txt}  Duration: {duration_txt}  Cumulative: {cumul_txt}");
		Paragraph::new(header_line).render(header_a, buf);

		// -- Draw Logs
		let logs = if let Some(current_run) = state.current_run() {
			LogBmc::list_for_display(state.mm(), current_run.id)
		} else {
			Ok(Vec::new())
		};

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
		StatefulWidget::render(list_w, logs_a.x_margin(1), buf, &mut list_s);
	}
}
