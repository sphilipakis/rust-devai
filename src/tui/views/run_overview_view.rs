use crate::tui::AppState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget as _};

/// Placeholder view for *Before All* tab.
pub struct RunOverviewView;

impl StatefulWidget for RunOverviewView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
		Paragraph::new("Run Overview").render(area, buf);
	}
}
