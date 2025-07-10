use crate::tui::core::AppState;
use crate::tui::styles;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};

pub struct ActionView;

impl StatefulWidget for ActionView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// Block::new().render(area, buf);

		let n_label = if state.show_runs() {
			"] Hide Runs Nav"
		} else {
			"] Show Runs Nav"
		};

		let line = Line::from(vec![
			Span::raw("["),
			Span::styled("r", styles::STL_TXT_ACTION),
			Span::raw("] Replay  "),
			Span::raw("["),
			Span::styled("q", styles::STL_TXT_ACTION),
			Span::raw("] Quit  "),
			Span::raw("["),
			Span::styled("n", styles::STL_TXT_ACTION),
			Span::raw(n_label),
		]);

		Paragraph::new(line).render(area, buf);
	}
}
