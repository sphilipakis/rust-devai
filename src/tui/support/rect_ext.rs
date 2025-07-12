use ratatui::layout::Rect;

/// Convenient Ratatui Area/Rect utility functions
///
/// NOTE: Here some of those function does not use Layout
///       (we might change that later to use Ratatui functions)
#[allow(unused)]
pub trait RectExt {
	// -- marings
	fn x_margin(&self, margin: u16) -> Rect;
	fn x_h_margin(&self, h_margin: u16) -> Rect;
	#[allow(unused)]
	fn x_v_margin(&self, v_margin: u16) -> Rect;

	fn x_move_top(&self, y: u16) -> Rect;

	// -- lines
	fn x_line(&self, line_idx: u16) -> Rect;

	// -- Width & Height
	fn x_width(&self, width: u16) -> Rect;
	fn x_height(&self, height: u16) -> Rect;
}

impl RectExt for Rect {
	// region:    --- Margins

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

	// endregion: --- Margins

	fn x_move_top(&self, y_offset: u16) -> Rect {
		Rect {
			x: self.x,
			y: self.y + y_offset,
			width: self.width,
			height: self.height,
		}
	}

	// region:    --- Width & Height

	// region:    --- Lines

	// - `line_num` starts at ``
	fn x_line(&self, line_num: u16) -> Rect {
		Rect {
			x: self.x,
			y: self.y + line_num - 1,
			width: self.width,
			height: 1,
		}
	}
	// endregion: --- Lines

	/// Change the width
	fn x_width(&self, width: u16) -> Rect {
		Rect {
			x: self.x,
			y: self.y,
			width,
			height: self.height,
		}
	}

	// change the height
	fn x_height(&self, height: u16) -> Rect {
		Rect {
			x: self.x,
			y: self.y,
			width: self.width,
			height,
		}
	}
	// endregion: --- Width & Height
}

// alternatively
// ratatui::layout::Layout::default()
// 	.horizontal_margin(h_margin)
// 	.constraints(vec![ratatui::layout::Constraint::Fill(1)])
// 	.split(*self)[0]
