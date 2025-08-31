use crate::Result;
use crate::support::os;
use crate::term::safer_println;
use crossterm::{
	execute,
	style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::collections::HashSet;
use std::io::{Write, stdout};

/// Prints the status of API keys, indicating which are available and which are missing.
pub fn print_api_keys(all_keys: &[&str], set_keys: &HashSet<String>) -> Result<()> {
	let mut stdout = stdout();
	let mut set_keys_list = Vec::new();
	let mut other_keys_list = Vec::new();

	// Separate keys into available and missing lists
	for &key in all_keys {
		if set_keys.contains(key) {
			set_keys_list.push(key);
		} else {
			other_keys_list.push(key);
		}
	}

	// --- Print Set Keys
	if !set_keys_list.is_empty() {
		writeln!(stdout, "\nAvailable API Keys:\n")?;
		for key in set_keys_list {
			execute!(
				stdout,
				SetForegroundColor(Color::Green),
				Print("✔  "),
				ResetColor,
				Print(key),
				Print("\n")
			)?;
		}

		let other_keys_strs = other_keys_list.join(", ");
		writeln!(stdout, "\nOther possible API Keys: {other_keys_strs}")?;
	}
	// --- If no set keys, then, warning like message
	else {
		writeln!(stdout, "\nNo API Keys Set\n")?;
		for key in other_keys_list {
			execute!(
				stdout,
				SetForegroundColor(Color::Red),
				Print("✖  "),
				ResetColor,
				Print(key),
				Print("\n")
			)?;
		}
		writeln!(stdout, "\nSet at least one API key.\n")?;

		let help_message = os::get_set_api_key_message();

		safer_println(help_message, false);
	}

	writeln!(stdout)?;
	stdout.flush()?;

	Ok(())
}
