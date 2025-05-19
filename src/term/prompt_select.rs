use crate::Result;
use crossterm::{
	cursor,
	event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
	execute,
	style::{Color, Print, ResetColor, SetForegroundColor},
	terminal::{Clear, ClearType},
};
use std::io::{Stdout, Write};

pub fn prompt_select(stdout: &mut Stdout, prompt: &str, options: &[&str]) -> Result<usize> {
	let mut current_selection = 0;
	let num_options = options.len();

	loop {
		execute!(
			stdout,
			Clear(ClearType::All),
			cursor::MoveTo(0, 0),
			Print(prompt),
			cursor::MoveToNextLine(1)
		)?;

		for (i, option_text) in options.iter().enumerate() {
			if i == current_selection {
				execute!(
					stdout,
					SetForegroundColor(Color::Green),
					Print(format!("> {}. {}", i + 1, option_text)),
					ResetColor,
					cursor::MoveToNextLine(1)
				)?;
			} else {
				execute!(
					stdout,
					Print(format!("  {}. {}", i + 1, option_text)),
					cursor::MoveToNextLine(1)
				)?;
			}
		}
		stdout.flush()?;

		if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
			// Allow Ctrl+C to exit
			if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('c') {
				return Err(crate::Error::UserInterrupted);
			}

			match code {
				KeyCode::Up => {
					if current_selection > 0 {
						current_selection -= 1;
					} else {
						current_selection = num_options - 1; // Wrap around
					}
				}
				KeyCode::Down => {
					if current_selection < num_options - 1 {
						current_selection += 1;
					} else {
						current_selection = 0; // Wrap around
					}
				}
				KeyCode::Enter => {
					break Ok(current_selection);
				}
				KeyCode::Char(c) => {
					if let Some(digit) = c.to_digit(10) {
						if digit > 0 && digit <= num_options as u32 {
							current_selection = (digit - 1) as usize;
							break Ok(current_selection);
						}
					}
				}
				KeyCode::Esc => {
					return Err(crate::Error::UserInterrupted);
				}
				_ => {}
			}
		}
	}
}
