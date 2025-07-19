use super::{RunMainView, RunsNavView};
use crate::tui::AppState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::StatefulWidget;

pub struct RunsView;

impl StatefulWidget for RunsView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- Layout Nav | Content
		// Empty line on top
		let [area] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![Constraint::Fill(1)])
			.areas(area);

		let [nav_a, main_a] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![Constraint::Max(20), Constraint::Fill(1)])
			.spacing(1)
			.areas(area);

		// -- Render nav
		// IMPORTANT: Need to render this one first, as it will update run_idx
		RunsNavView.render(nav_a, buf, state);

		if state.should_redraw() {
			return;
		}

		// -- Display the Content block
		RunMainView.render(main_a, buf, state);
	}
}
