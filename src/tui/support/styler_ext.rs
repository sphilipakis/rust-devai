use ratatui::style::Color;
use ratatui::text::Span;

pub trait StylerExt {
	fn x_bg(self, color: Color) -> Self;
}

impl StylerExt for Vec<Span<'_>> {
	fn x_bg(mut self, color: Color) -> Self {
		for span in self.iter_mut() {
			span.style.bg = color.into();
		}
		self
	}
}
