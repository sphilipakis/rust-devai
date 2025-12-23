use crate::Result;
use simple_fs::SPath;
use std::env;
use std::process::Command;

// region:    --- EditorProgram

/// Known editor programs that can be auto-detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorProgram {
	Zed,
	Vscode,
	Neovim,
	Vim,
	Emacs,
	Nano,
	Sublime,
	Atom,
	Custom(String),
}

impl EditorProgram {
	/// Returns the command name to invoke this editor
	pub fn program(&self) -> &str {
		match self {
			EditorProgram::Zed => "zed",
			EditorProgram::Vscode => "code",
			EditorProgram::Neovim => "nvim",
			EditorProgram::Vim => "vim",
			EditorProgram::Emacs => "emacs",
			EditorProgram::Nano => "nano",
			EditorProgram::Sublime => "subl",
			EditorProgram::Atom => "atom",
			EditorProgram::Custom(name) => name.as_str(),
		}
	}

	/// Parse an editor program from a string (command name or program name).
	/// Returns a known variant if recognized, otherwise Custom.
	pub fn from_str(s: &str) -> EditorProgram {
		let s_lower = s.to_lowercase();
		match s_lower.as_str() {
			"zed" => EditorProgram::Zed,
			"code" | "vscode" => EditorProgram::Vscode,
			"nvim" | "neovim" => EditorProgram::Neovim,
			"vim" | "vi" => EditorProgram::Vim,
			"emacs" => EditorProgram::Emacs,
			"nano" => EditorProgram::Nano,
			"subl" | "sublime" | "sublime_text" => EditorProgram::Sublime,
			"atom" => EditorProgram::Atom,
			_ => EditorProgram::Custom(s.to_string()),
		}
	}
}

// endregion: --- EditorProgram

// region:    --- Public Functions

/// Opens a file in the auto-detected editor.
/// Returns the editor program if successful, or Error if no editor was found.
pub fn open_file_auto(path: &SPath) -> Result<EditorProgram> {
	let Some(editor) = editor_program() else {
		return Err(
			format!("No editor found. Cannot open '{path}'.\nSet your VISUAL or EDITOR environment variable.").into(),
		);
	};

	let program = editor.program();
	Command::new(program)
		.arg(path.as_str())
		.spawn()
		.map_err(|err| format!("Failed to open editor '{program}' for file '{path}'.\nCause: {err}"))?;

	Ok(editor)
}

/// Returns the detected editor program based on environment variables.
/// Detection order:
/// 1. Integrated terminal settings (ZED_TERM, TERM_PROGRAM)
/// 2. Standard environment variables (VISUAL, EDITOR)
pub fn editor_program() -> Option<EditorProgram> {
	if let Some(editor) = find_integrated_term_editor() {
		return Some(editor);
	}

	if let Some(editor) = find_standard_env_term_editor() {
		return Some(editor);
	}

	None
}

// endregion: --- Public Functions

// region:    --- Support

/// Finds an editor based on integrated terminal environment variables.
fn find_integrated_term_editor() -> Option<EditorProgram> {
	// -- Check TERM_PROGRAM
	if let Ok(term_program) = env::var("TERM_PROGRAM") {
		let term_lower = term_program.to_lowercase();

		if term_lower.contains("zed") {
			return Some(EditorProgram::Zed);
		}
		if term_lower.contains("vscode") || term_lower.contains("code") {
			return Some(EditorProgram::Vscode);
		}
		if term_lower.contains("nvim") || term_lower.contains("neovim") {
			return Some(EditorProgram::Neovim);
		}
		if term_lower.contains("vim") {
			return Some(EditorProgram::Vim);
		}
		if term_lower.contains("emacs") {
			return Some(EditorProgram::Emacs);
		}
		if term_lower.contains("sublime") {
			return Some(EditorProgram::Sublime);
		}
		if term_lower.contains("atom") {
			return Some(EditorProgram::Atom);
		}
	}

	// -- Check ZED_TERM
	// NOTE: This one is tricky. If call vscode from zed term, then this will be true.
	//       Theis is why do last.
	if let Ok(zed_term) = env::var("ZED_TERM")
		&& (zed_term.eq_ignore_ascii_case("true") || zed_term == "1")
	{
		return Some(EditorProgram::Zed);
	}

	None
}

/// Finds an editor based on standard environment variables (VISUAL, EDITOR).
fn find_standard_env_term_editor() -> Option<EditorProgram> {
	// -- Check VISUAL
	if let Ok(visual) = env::var("VISUAL")
		&& !visual.is_empty()
	{
		let program = extract_program_name(&visual);
		return Some(EditorProgram::from_str(&program));
	}

	// -- Check EDITOR
	if let Ok(editor) = env::var("EDITOR")
		&& !editor.is_empty()
	{
		let program = extract_program_name(&editor);
		return Some(EditorProgram::from_str(&program));
	}

	None
}

/// Extracts the program name from a path or command string
fn extract_program_name(path_or_cmd: &str) -> String {
	// Handle cases like "/usr/bin/vim" or "vim -u NONE"
	let first_part = path_or_cmd.split_whitespace().next().unwrap_or(path_or_cmd);

	// Extract the filename from the path
	if let Some(name) = first_part.rsplit('/').next() {
		if let Some(name) = name.rsplit('\\').next() {
			return name.to_string();
		}
		return name.to_string();
	}

	first_part.to_string()
}

// endregion: --- Support
