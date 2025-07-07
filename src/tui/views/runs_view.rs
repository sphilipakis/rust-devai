use super::{RunContentView, RunsNavView};
use crate::tui::AppState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::StatefulWidget;

pub struct RunsView {}

impl StatefulWidget for RunsView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- Layout Nav | Content
		// Empty line on top
		let [_, area] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![Constraint::Length(1), Constraint::Fill(1)])
			.areas(area);

		let [nav_a, content_a] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![Constraint::Max(20), Constraint::Fill(1)])
			.spacing(1)
			.areas(area);

		// -- Render nav
		let runs_nav_v = RunsNavView {};
		runs_nav_v.render(nav_a, buf, state);

		// -- Display the Content block
		let run_content_v = RunContentView {};
		run_content_v.render(content_a, buf, state);
	}
}
