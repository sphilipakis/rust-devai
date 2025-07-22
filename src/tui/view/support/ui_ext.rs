use ratatui::style::Color;
use ratatui::text::Span;

pub trait UiExt {
	/// Sent the bg for all ui elements
	fn x_bg(self, color: Color) -> Self;

	/// Sent the fg for all ui elements
	fn x_fg(self, color: Color) -> Self;

	// Total width of all ui elements
	fn x_width(&self) -> u16;
}

impl UiExt for Vec<Span<'_>> {
	fn x_bg(mut self, color: Color) -> Self {
		for span in self.iter_mut() {
			span.style.bg = color.into();
		}
		self
	}

	fn x_fg(mut self, color: Color) -> Self {
		for span in self.iter_mut() {
			span.style.fg = color.into();
		}
		self
	}

	fn x_width(&self) -> u16 {
		self.iter().map(|span| span.width() as u16).sum()
	}
}
