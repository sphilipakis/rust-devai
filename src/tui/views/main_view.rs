use super::{ActionView, RunsView, SumView};
use crate::tui::AppState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, StatefulWidget, Widget};

pub struct MainView {}

impl StatefulWidget for MainView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- Add background
		Block::new().on_black().render(area, buf);

		// -- Layout
		let [header_a, runs_a, action_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![Constraint::Length(2), Constraint::Fill(1), Constraint::Length(1)])
			.areas(area);

		// -- Render header
		let sum_v = SumView {};
		sum_v.render(header_a, buf, state);

		// -- Render main
		let run_v = RunsView {};
		run_v.render(runs_a, buf, state);

		// -- Render action
		let action_v = ActionView {};
		action_v.render(action_a, buf);
	}
}
