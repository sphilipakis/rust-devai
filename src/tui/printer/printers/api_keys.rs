use crate::{Result, support::os, tui::support::safer_println};
use crossterm::{
	execute,
	style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::collections::HashSet;
use std::io::{Write, stdout};

/// Prints the status of API keys, indicating which are available and which are missing.
pub fn print_api_keys(all_keys: &[&str], available_keys: &HashSet<String>) -> Result<()> {
	let mut stdout = stdout();
	let mut available_list = Vec::new();
	let mut missing_list = Vec::new();

	// Separate keys into available and missing lists
	for &key in all_keys {
		if available_keys.contains(key) {
			available_list.push(key);
		} else {
			missing_list.push(key);
		}
	}

	// --- Print Available Keys
	if !available_list.is_empty() {
		writeln!(stdout, "\nAPI Keys available:\n")?;
		for key in available_list {
			execute!(
				stdout,
				SetForegroundColor(Color::Green),
				Print("✔  "),
				ResetColor,
				Print(key),
				Print("\n")
			)?;
		}
	}

	// --- Print Missing Keys
	if !missing_list.is_empty() {
		writeln!(stdout, "\nAPI Keys missing:\n")?;
		for key in missing_list {
			execute!(
				stdout,
				SetForegroundColor(Color::Red),
				Print("✖  "),
				ResetColor,
				Print(key),
				Print("\n")
			)?;
		}
	}

	writeln!(stdout)?;
	stdout.flush()?;

	let help_message = os::get_set_api_key_message();

	safer_println(help_message, false);

	Ok(())
}
