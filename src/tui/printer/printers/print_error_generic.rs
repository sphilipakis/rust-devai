use crate::tui::support::safer_println;
use crossterm::{
	cursor::MoveToColumn,
	execute,
	style::{Color, Print, ResetColor, SetForegroundColor},
	terminal::{Clear, ClearType},
};
use std::io::stdout;

#[allow(unused_must_use)] // TODO: need to remove and make this function return error
pub fn print_error_generic(msg: &str, interactive: bool) {
	let mut stdout = stdout();

	execute!(
		stdout,
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		SetForegroundColor(Color::Red),
		Print("\n======== Error\n\n"),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		ResetColor,
	);
	safer_println(msg, interactive);
	execute!(
		stdout,
		Print("\n\n"),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		SetForegroundColor(Color::Red),
		Print("=============\n"),
		ResetColor,
	);
}
