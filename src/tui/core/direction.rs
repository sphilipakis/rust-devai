use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum NavDir {
	Up,
	ShiftUp,
	Down,
	ShiftDown,
}

/// Constructors
impl NavDir {
	pub fn from_up_down_key_code(up: KeyCode, down: KeyCode, key_event: Option<&KeyEvent>) -> Option<NavDir> {
		let key_event = key_event?;
		let key_code = key_event.code;
		let has_shift = key_event.modifiers.contains(KeyModifiers::SHIFT);

		if up == key_code {
			if has_shift {
				Some(NavDir::ShiftUp)
			} else {
				Some(NavDir::Up)
			}
		} else if down == key_code {
			if has_shift {
				Some(NavDir::ShiftDown)
			} else {
				Some(NavDir::Down)
			}
		} else {
			None
		}
	}
}

/// Accessors
#[allow(unused)]
impl NavDir {
	// up = -1, down = 1
	pub fn offset(&self) -> i32 {
		match self {
			NavDir::Up | NavDir::ShiftUp => -1,
			NavDir::Down | NavDir::ShiftDown => 1,
		}
	}

	pub fn is_up(&self) -> bool {
		matches!(self, NavDir::Up | NavDir::ShiftUp)
	}
	pub fn is_down(&self) -> bool {
		matches!(self, NavDir::Down | NavDir::ShiftDown)
	}
	#[allow(unused)]
	pub fn is_shifted(&self) -> bool {
		matches!(self, NavDir::ShiftUp | NavDir::ShiftDown)
	}
}
