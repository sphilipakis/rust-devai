use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};

/// The application mouse event
/// Small wrapper on crossterm mouse event with some application specific methods.
#[derive(Debug, Clone, Copy)]
pub struct MouseEvt {
	mouse_event: MouseEvent,
}

impl MouseEvt {
	#[allow(unused)]
	pub fn x(&self) -> u16 {
		self.mouse_event.column
	}
	pub fn y(&self) -> u16 {
		self.mouse_event.row
	}
	pub fn position(&self) -> Position {
		Position::new(self.mouse_event.column, self.mouse_event.row)
	}

	pub fn is_over(&self, area: Rect) -> bool {
		area.contains(self.position())
	}

	pub fn is_move(&self) -> bool {
		match self.mouse_event.kind {
			MouseEventKind::Moved => true,
			// -- everything else false for now (to see the options quickly)
			MouseEventKind::Down(_) => false,
			MouseEventKind::Up(_) => false,
			MouseEventKind::Drag(_) => false,
			MouseEventKind::ScrollDown => false,
			MouseEventKind::ScrollUp => false,
			MouseEventKind::ScrollLeft => false,
			MouseEventKind::ScrollRight => false,
		}
	}

	pub fn is_down(&self) -> bool {
		match self.mouse_event.kind {
			// -- everything else false for now
			MouseEventKind::Down(_) => true,
			_ => false,
		}
	}

	pub fn is_up(&self) -> bool {
		match self.mouse_event.kind {
			// -- everything else false for now
			MouseEventKind::Up(_) => true,
			_ => false,
		}
	}
}

// region:    --- Froms

impl From<&MouseEvent> for MouseEvt {
	fn from(mouse_event: &MouseEvent) -> Self {
		Self {
			mouse_event: *mouse_event,
		}
	}
}

impl From<MouseEvent> for MouseEvt {
	fn from(mouse_event: MouseEvent) -> Self {
		Self { mouse_event }
	}
}

// endregion: --- Froms

// region:    --- Intos

impl From<MouseEvt> for Position {
	fn from(value: MouseEvt) -> Self {
		value.position()
	}
}

impl From<&MouseEvt> for Position {
	fn from(value: &MouseEvt) -> Self {
		value.position()
	}
}

// endregion: --- Intos
