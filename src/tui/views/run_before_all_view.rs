use crate::tui::AppState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget as _};

/// Placeholder view for *Before All* tab.
pub struct RunBeforeAllView {}

impl StatefulWidget for RunBeforeAllView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
		Paragraph::new("Before All content").render(area, buf);
	}
}
