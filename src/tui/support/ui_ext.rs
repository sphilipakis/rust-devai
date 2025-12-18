use ratatui::style::Color;
use ratatui::text::{Line, Span};

pub trait UiExt {
	/// Sent the bg for all ui elements
	fn x_bg(self, color: Color) -> Self;

	/// Sent the fg for all ui elements
	#[allow(unused)]
	fn x_fg(self, color: Color) -> Self;

	// Total width of all ui elements
	fn x_width(&self) -> u16;
}

impl UiExt for Line<'_> {
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

impl<'a> UiExt for &mut [Span<'a>] {
	fn x_bg(self, color: Color) -> Self {
		for span in self.iter_mut() {
			span.style.bg = color.into();
		}
		self
	}

	fn x_fg(self, color: Color) -> Self {
		for span in self.iter_mut() {
			span.style.fg = color.into();
		}
		self
	}

	fn x_width(&self) -> u16 {
		self.iter().map(|span| span.width() as u16).sum()
	}
}

impl<'a> UiExt for Option<&mut [Span<'a>]> {
	fn x_bg(self, color: Color) -> Self {
		if let Some(slice) = self {
			for span in slice.iter_mut() {
				span.style.bg = color.into();
			}
			Some(slice)
		} else {
			None
		}
	}

	fn x_fg(self, color: Color) -> Self {
		if let Some(slice) = self {
			for span in slice.iter_mut() {
				span.style.fg = color.into();
			}
			Some(slice)
		} else {
			None
		}
	}

	fn x_width(&self) -> u16 {
		match self {
			Some(slice) => slice.iter().map(|span| span.width() as u16).sum(),
			None => 0,
		}
	}
}

/// IMPORTANT: This cannot mutate states, just the x_width will work
/// TODO: Might want to split those functions in different ext
impl<'a> UiExt for &[Span<'a>] {
	fn x_bg(self, _color: Color) -> Self {
		// Cannot mutate, so do nothing and return self
		self
	}

	fn x_fg(self, _color: Color) -> Self {
		// Cannot mutate, so do nothing and return self
		self
	}

	fn x_width(&self) -> u16 {
		self.iter().map(|span| span.width() as u16).sum()
	}
}
