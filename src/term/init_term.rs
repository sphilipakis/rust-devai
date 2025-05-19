use crate::Result;
use crossterm::{cursor, execute, terminal};
use std::io::Stdout;

pub fn init_term() -> Result<Stdout> {
	let mut stdout = std::io::stdout();
	terminal::enable_raw_mode()?;
	execute!(stdout, cursor::Hide)?;

	Ok(stdout)
}
