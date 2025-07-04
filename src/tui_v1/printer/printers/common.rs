use crossterm::{
	cursor, execute,
	style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
	terminal::{self, ClearType},
};
use std::io::stdout;

#[allow(unused)]
pub fn print_blue_label_white_message(label: &str, msg: &str, interactive: bool) {
	let stdout = stdout();
	let mut stdout_lock = stdout.lock(); // Locking stdout to avoid multiple open handles

	let lines: Vec<&str> = if interactive {
		msg.split("\n").collect()
	} else {
		vec![msg]
	};

	let _ = execute!(
		stdout_lock,
		terminal::Clear(ClearType::CurrentLine),
		cursor::MoveToColumn(0),
		SetAttribute(Attribute::Bold),
		SetForegroundColor(Color::Blue),
		Print(label),
	);

	for (idx, line) in lines.iter().enumerate() {
		if idx > 0 && interactive {
			let _ = execute!(
				stdout_lock,
				terminal::Clear(ClearType::CurrentLine),
				cursor::MoveToColumn(0)
			);
		}
		// TODO: need to handle error
		let _ = execute!(
			stdout_lock,
			ResetColor,
			SetAttribute(Attribute::Reset),
			Print(format!("{line}\n")),
			ResetColor,
			SetAttribute(Attribute::Reset)
		);
	}
}
