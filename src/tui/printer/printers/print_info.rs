use crate::Result;
use crossterm::{
	cursor::MoveToColumn,
	execute,
	style::{Color, Print, ResetColor, SetForegroundColor},
	terminal::{Clear, ClearType},
};
use std::io::{Write as _, stdout};

#[allow(unused_must_use)] // TODO: need to remove and make this function return error
pub fn print_info_short(msg: &str) -> Result<()> {
	let mut stdout = stdout();

	execute!(
		stdout,
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		SetForegroundColor(Color::Blue),
		Print("Info "),
		SetForegroundColor(Color::White),
		Print(msg),
		ResetColor,
	);

	writeln!(stdout)?;
	stdout.flush()?;

	Ok(())
}
