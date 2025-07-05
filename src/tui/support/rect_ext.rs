use ratatui::layout::Rect;

pub trait RectExt {
	fn x_margin(&self, margin: u16) -> Rect;
	fn x_h_margin(&self, h_margin: u16) -> Rect;
	#[allow(unused)]
	fn x_v_margin(&self, v_margin: u16) -> Rect;
}

/// NOTE: Here some of those function does not use Layout
///       And provide direction compute (we might change that later)
impl RectExt for Rect {
	// Margin on all sides
	fn x_margin(&self, margin: u16) -> Rect {
		let x = (self.x + margin).min(self.x + self.width);
		let y = (self.y + margin).min(self.y + self.height);
		let width = self.width.saturating_sub(2 * margin);
		let height = self.height.saturating_sub(2 * margin);

		Rect { x, y, width, height }
	}

	// Only horizontal margin
	fn x_h_margin(&self, h_margin: u16) -> Rect {
		let x = (self.x + h_margin).min(self.x + self.width);
		let width = self.width.saturating_sub(2 * h_margin);

		Rect {
			x,
			y: self.y,
			width,
			height: self.height,
		}
	}

	fn x_v_margin(&self, v_margin: u16) -> Rect {
		let y = (self.y + v_margin).min(self.y + self.height);
		let height = self.height.saturating_sub(2 * v_margin);

		Rect {
			x: self.x,
			y,
			width: self.width,
			height,
		}
	}
}

// alternatively
// ratatui::layout::Layout::default()
// 	.horizontal_margin(h_margin)
// 	.constraints(vec![ratatui::layout::Constraint::Fill(1)])
// 	.split(*self)[0]
