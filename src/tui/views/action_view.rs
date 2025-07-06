use crate::tui::styles::STL_TXT_ACTION;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub struct ActionView;

impl Widget for ActionView {
	fn render(self, area: Rect, buf: &mut Buffer) {
		// Block::new().render(area, buf);

		let line = Line::from(vec![
			Span::raw("["),
			Span::styled("r", STL_TXT_ACTION),
			Span::raw("] Replay  "),
			Span::raw("["),
			Span::styled("q", STL_TXT_ACTION),
			Span::raw("] Quit  "),
			Span::raw("["),
			Span::styled("a", STL_TXT_ACTION),
			Span::raw("] Open Agent"),
		]);

		Paragraph::new(line).render(area, buf);
	}
}
