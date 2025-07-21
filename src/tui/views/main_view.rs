use super::{ActionView, RunsView, SumView};
use crate::tui::views::RunMainView;
use crate::tui::{AppState, styles};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, StatefulWidget, Widget};

pub struct MainView {}

impl StatefulWidget for MainView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- Add background
		Block::new().bg(styles::CLR_BKG_BLACK).render(area, buf);

		// -- Layout
		let [header_a, _gap_a, content_a, action_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(1), // Header line
				Constraint::Length(1), // Gap
				Constraint::Fill(1),   // content
				Constraint::Length(1), // Action bar
			])
			.areas(area);

		// -- Render header
		SumView.render(header_a, buf, state);

		// -- Render main
		if state.show_runs() {
			RunMainView::clear_scroll_idens(state);
			RunsView.render(content_a, buf, state);
		} else {
			RunsView::clear_scroll_idens(state);
			RunMainView.render(content_a, buf, state);
		}

		// -- Render action
		let action_v = ActionView {};
		action_v.render(action_a, buf, state);
	}
}
