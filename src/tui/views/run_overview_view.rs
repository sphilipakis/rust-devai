use crate::tui::AppState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Padding, Paragraph, StatefulWidget, Widget as _};

/// Placeholder view for *Before All* tab.
pub struct RunOverviewView;

impl StatefulWidget for RunOverviewView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
		let p = Paragraph::new("Run Overview").block(Block::new().padding(Padding::uniform(1)));

		p.render(area, buf);
	}
}
