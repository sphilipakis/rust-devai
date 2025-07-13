use crate::support::text::format_duration_us;
use crate::tui::support;
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};

pub struct SumView;

impl StatefulWidget for SumView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- Layout
		let [lbl_1, val_1, lbl_2, val_2, lbl_3, val_3] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![
				//
				Constraint::Length(11), // Total Runs
				Constraint::Length(6),
				Constraint::Length(11), // Cost
				Constraint::Length(9),
				Constraint::Length(15), // Duration
				Constraint::Length(15),
			])
			.spacing(1)
			.areas(area);

		Block::new().bg(styles::CLR_BKG_BLACK).render(area, buf);

		// -- Extract Data
		let mut duration_us: i64 = 0;
		let mut cost: Option<f64> = None;
		let mut done_runs_count = 0;
		for run in state.runs() {
			run.is_done().then(|| done_runs_count += 1);
			if let Some(run_cost) = run.total_cost {
				// cost += run_cost
				cost = Some(cost.unwrap_or(0.0) + run_cost);
			};
			if let (Some(start), Some(end)) = (run.start, run.end) {
				duration_us += end.as_i64() - start.as_i64();
			}
		}
		let running_run = state.runs().len() - done_runs_count;

		// -- Format Data
		let mut runs_fmt = done_runs_count.to_string();
		if running_run > 0 {
			runs_fmt = format!("{runs_fmt} ({running_run})")
		}
		let duration_fmt = format_duration_us(duration_us);
		let cost_fmt = support::ui_fmt_cost(cost);

		// -- Render
		Paragraph::new("Total Runs:")
			.style(styles::STL_FIELD_LBL_DARK)
			.right_aligned()
			.render(lbl_1, buf);
		Paragraph::new(runs_fmt).style(styles::STL_FIELD_VAL_DARK).render(val_1, buf);

		Paragraph::new("Total Cost:")
			.style(styles::STL_FIELD_LBL_DARK)
			.right_aligned()
			.render(lbl_2, buf);
		Paragraph::new(cost_fmt).style(styles::STL_FIELD_VAL_DARK).render(val_2, buf);

		Paragraph::new("Total Duration:")
			.style(styles::STL_FIELD_LBL_DARK)
			.right_aligned()
			.render(lbl_3, buf);
		Paragraph::new(duration_fmt)
			.style(styles::STL_FIELD_VAL_DARK)
			.render(val_3, buf);
	}
}
