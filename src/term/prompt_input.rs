use crate::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute, queue};
use std::io::{Stdout, Write};

pub fn prompt_input(stdout: &mut Stdout, prompt_text: &str, clear: bool) -> Result<String> {
	let mut input = String::new();
	let (_current_col, current_row) = cursor::position()?;

	let start_row = if clear { 0 } else { current_row };

	loop {
		// let input_prefix = if input.is_empty() { " |" } else { " " };
		queue!(stdout, cursor::MoveTo(0, start_row))?;

		if clear {
			queue!(stdout, Clear(ClearType::FromCursorDown))?;
		}
		queue!(
			stdout,
			Print(prompt_text),
			Print(" "),
			Print(&input),
			Print("█"),
			cursor::MoveTo(prompt_text.len() as u16 + input.len() as u16, start_row)
		)?;
		stdout.flush()?;

		if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
			if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('c') {
				return Err(crate::Error::UserInterrupted);
			}
			match code {
				KeyCode::Char(c) => {
					input.push(c);
				}
				KeyCode::Backspace => {
					input.pop();
					// remove caret
					execute!(
						stdout,
						// need to +1, because we have th cursor::MoveTo above at the length of the text
						cursor::MoveRight(1),
						Print(" "),
					)?;
				}
				KeyCode::Enter => {
					let trimmed_input = input.trim();
					if !trimmed_input.is_empty() {
						execute!(
							stdout,
							// need to +1, because we have th cursor::MoveTo above at the length of the text
							cursor::MoveRight(1),
							Print(" "),
							cursor::MoveTo(0, start_row + 1),
							Clear(ClearType::CurrentLine),
							ResetColor,
						)?;
						stdout.flush()?;

						break Ok(trimmed_input.to_string());
					} else {
						execute!(
							stdout,
							cursor::MoveTo(0, start_row + 1),
							Clear(ClearType::CurrentLine),
							SetForegroundColor(Color::Red),
							Print("Agent name cannot be empty. Press any key to try again."),
							ResetColor
						)?;
						stdout.flush()?;
						event::read()?;
						execute!(stdout, cursor::MoveTo(0, start_row + 1), Clear(ClearType::CurrentLine))?;
					}
				}
				KeyCode::Esc => {
					return Err(crate::Error::Custom("Input cancelled by user".to_string()));
				}
				_ => {}
			}
		}
	}
}
