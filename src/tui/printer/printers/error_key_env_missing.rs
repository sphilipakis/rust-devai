use crossterm::{
	cursor::MoveToColumn,
	execute,
	style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
	terminal::{Clear, ClearType},
};
use genai::ModelIden;
use std::io::stdout;

#[allow(unused_must_use)] // TODO: need to remove and make this function return error
pub fn print_key_env_missing(missing_env_name: &str, model_iden: &ModelIden, _interactive: bool) {
	let mut stdout = stdout();

	let model_name: &str = &model_iden.model_name;
	let provider_name: &str = model_iden.adapter_kind.as_str();

	execute!(
		stdout,
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		SetForegroundColor(Color::Red),
		Print("\n======== Environment Error\n\n"),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		ResetColor,
		SetForegroundColor(Color::Red),
		SetAttribute(Attribute::Bold),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		Print("Error: "),
		ResetColor,
		Print("Cannot connect to model "),
		SetForegroundColor(Color::Yellow),
		Print("'"),
		Print(model_name),
		Print("'"),
		ResetColor,
		Print(" for provider "),
		SetForegroundColor(Color::Yellow),
		Print("'"),
		Print(provider_name),
		Print("'."),
		ResetColor,
		SetForegroundColor(Color::Red),
		SetAttribute(Attribute::Bold),
		Print("\n"),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		Print("Cause: "),
		ResetColor,
		Print("Environment variable "),
		SetForegroundColor(Color::Magenta),
		Print(format!("'{}'", missing_env_name)),
		ResetColor,
		Print(" missing. Make sure to set it for this terminal."),
		SetAttribute(Attribute::Bold),
		Print("\n"),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		Print("\n"),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		Print("You can set environment variable like:"),
		Print("\n"),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		Print("\n"),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		ResetColor,
		SetForegroundColor(Color::Green),
		Print(format!(
			"export {}=\"YOUR_{}_VALUE_HERE\"\n",
			missing_env_name, missing_env_name
		)),
		Print("\n"),
		Clear(ClearType::CurrentLine),
		MoveToColumn(0),
		SetForegroundColor(Color::Red),
		Print("========================\n"),
		ResetColor,
	);
}
