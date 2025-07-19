use ratatui::style::Color;
use ratatui::text::Span;

pub trait UiExt {
	fn x_bg(self, color: Color) -> Self;
	fn x_total_width(&self) -> u16;
}

impl UiExt for Vec<Span<'_>> {
	fn x_bg(mut self, color: Color) -> Self {
		for span in self.iter_mut() {
			span.style.bg = color.into();
		}
		self
	}

	fn x_total_width(&self) -> u16 {
		self.iter().map(|span| span.width() as u16).sum()
	}
}
