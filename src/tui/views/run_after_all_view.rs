use crate::tui::AppState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget as _};

/// Placeholder view for *After All* tab.
pub struct RunAfterAllView {}

impl StatefulWidget for RunAfterAllView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
		Paragraph::new("After All content").render(area, buf);
	}
}
