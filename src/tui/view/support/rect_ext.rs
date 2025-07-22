use ratatui::layout::Rect;

/// Convenient Ratatui Area/Rect utility functions
///
/// NOTE: Here some of those function does not use Layout
///       (we might change that later to use Ratatui functions)
#[allow(unused)]
pub trait RectExt {
	// -- Margins
	fn x_margin(&self, margin: u16) -> Rect;
	fn x_h_margin(&self, h_margin: u16) -> Rect;
	#[allow(unused)]
	fn x_v_margin(&self, v_margin: u16) -> Rect;

	fn x_move_down(&self, y: u16) -> Rect;

	// -- Shrink
	/// Add to the y of the area, and remove the same from the .height
	fn x_shrink_from_top(&self, height_to_remove: u16) -> Rect;
	fn x_shrink_left(&self, width: u16) -> Rect;

	// -- Lines
	fn x_row(&self, row_num: u16) -> Rect;

	// -- Placement
	fn x_top_right(&self, width: u16, height: u16) -> Rect;
	fn x_bottom_right(&self, width: u16, height: u16) -> Rect;

	// -- Width & Height
	fn x_with_x(&self, x: u16) -> Rect;
	fn x_with_y(&self, y: u16) -> Rect;
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

	// region:    --- Shrink

	fn x_shrink_from_top(&self, height_to_remove: u16) -> Rect {
		let new_height = self.height.saturating_sub(height_to_remove);
		Rect {
			x: self.x,
			y: self.y + height_to_remove,
			width: self.width,
			height: new_height,
		}
	}

	fn x_shrink_left(&self, width: u16) -> Rect {
		let new_width = self.width.saturating_sub(width);
		let x = self.x + width;
		Rect {
			x,
			y: self.y,
			width: new_width,
			height: self.height,
		}
	}

	// endregion: --- Shrink

	fn x_move_down(&self, y_offset: u16) -> Rect {
		Rect {
			x: self.x,
			y: self.y + y_offset,
			width: self.width,
			height: self.height,
		}
	}

	// region:    --- Lines

	/// Make the area height 1, at the row level from this base area.
	/// - `row_num` starts at 1.
	fn x_row(&self, row_num: u16) -> Rect {
		Rect {
			x: self.x,
			y: self.y + row_num - 1,
			width: self.width,
			height: 1,
		}
	}
	// endregion: --- Lines

	// region:    --- Placement

	fn x_bottom_right(&self, width: u16, height: u16) -> Rect {
		Rect {
			x: self.x + self.width - width,
			y: self.y + self.height - height,
			width,
			height,
		}
	}

	fn x_top_right(&self, width: u16, height: u16) -> Rect {
		Rect {
			x: self.x + self.width - width,
			y: self.y,
			width,
			height,
		}
	}

	// endregion: --- Placement

	// region:    --- x,y,width,height

	/// Change the x position
	fn x_with_x(&self, x: u16) -> Rect {
		Rect {
			x,
			y: self.y,
			width: self.width,
			height: self.height,
		}
	}

	/// Change the y position
	fn x_with_y(&self, y: u16) -> Rect {
		Rect {
			x: self.x,
			y,
			width: self.width,
			height: self.height,
		}
	}

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
	// endregion: --- x,y,width,height
}

// alternatively
// ratatui::layout::Layout::default()
// 	.horizontal_margin(h_margin)
// 	.constraints(vec![ratatui::layout::Constraint::Fill(1)])
// 	.split(*self)[0]
