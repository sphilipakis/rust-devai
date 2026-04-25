use std::borrow::Cow;
use std::collections::HashSet;
use std::env;

// region:    --- TermProgram

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TermProgram {
	Zed,
	Vscode,
	WezTerm,
	Iterm,
	Alacritty,
	Tmux,
	Ghostty,
	Kitty,
	Warp,
	AppleTerminal,
	Xterm,
	Custom(String),
}

impl TermProgram {
	pub fn from_str(s: &str) -> Self {
		let s_lower = s.to_lowercase();
		match s_lower.as_str() {
			"vscode" => Self::Vscode,
			"zed" => Self::Zed,
			"wezterm" => Self::WezTerm,
			"iterm.app" | "iterm" => Self::Iterm,
			"apple_terminal" => Self::AppleTerminal,
			"alacritty" => Self::Alacritty,
			"tmux" => Self::Tmux,
			"ghostty" => Self::Ghostty,
			"kitty" => Self::Kitty,
			"warp" => Self::Warp,
			"xterm" | "xterm-256color" => Self::Xterm,
			_ => Self::Custom(s.to_string()),
		}
	}
}

// endregion: --- TermProgram

// region:    --- TermInfo

#[derive(Debug, Clone)]
pub struct TermInfo {
	pub term_program: Option<TermProgram>,
	pub term_variants: HashSet<TermProgram>,
}

impl TermInfo {
	pub fn match_any(&self, name: &str) -> bool {
		let target = TermProgram::from_str(name);
		self.term_program.as_ref() == Some(&target) || self.term_variants.contains(&target)
	}
}

impl Default for TermInfo {
	fn default() -> Self {
		let mut term_variants = HashSet::new();
		let mut term_program_opt = None;

		// -- Check TERM_PROGRAM
		if let Ok(val) = env::var("TERM_PROGRAM") {
			let prog = TermProgram::from_str(&val);
			term_variants.insert(prog.clone());
			term_program_opt = Some(prog);
		}

		// -- Loop once through env names to build the HashSet
		for (key, _) in env::vars() {
			let key_upper = key.to_uppercase();
			if key_upper == "TMUX" {
				term_variants.insert(TermProgram::Tmux);
			} else if key_upper.contains("ALACRITTY") {
				term_variants.insert(TermProgram::Alacritty);
			} else if key_upper.contains("VSCODE") {
				term_variants.insert(TermProgram::Vscode);
			} else if key_upper.contains("ZED") {
				term_variants.insert(TermProgram::Zed);
			} else if key_upper.contains("WEZTERM") {
				term_variants.insert(TermProgram::WezTerm);
			} else if key_upper.contains("ITERM") {
				term_variants.insert(TermProgram::Iterm);
			} else if key_upper.contains("GHOSTTY") {
				term_variants.insert(TermProgram::Ghostty);
			} else if key_upper.contains("KITTY") {
				term_variants.insert(TermProgram::Kitty);
			} else if key_upper.contains("WARP") {
				term_variants.insert(TermProgram::Warp);
			}
		}

		Self {
			term_program: term_program_opt,
			term_variants,
		}
	}
}

// endregion: --- TermInfo

// region:    --- TermTitleGuard

#[derive(Debug)]
pub struct TermTitleGuard {
	previous_title: Option<String>,
	restored: bool,
}

impl TermTitleGuard {
	pub fn capture() -> Self {
		Self {
			previous_title: tmux_pane_title(),
			restored: false,
		}
	}

	pub fn restore(mut self) -> bool {
		let restored = self.restore_inner();
		self.restored = true;
		restored
	}

	fn restore_inner(&self) -> bool {
		let Some(title) = &self.previous_title else {
			return false;
		};

		set_window_name(title)
	}
}

impl Drop for TermTitleGuard {
	fn drop(&mut self) {
		if !self.restored {
			let _ = self.restore_inner();
			self.restored = true;
		}
	}
}

// endregion: --- TermTitleGuard

// region:    --- Public Functions

pub fn term_info() -> Option<TermInfo> {
	let info = TermInfo::default();
	if info.term_program.is_some() || !info.term_variants.is_empty() {
		Some(info)
	} else {
		None
	}
}

pub fn set_window_name(name: &str) -> bool {
	let Some(term_info) = term_info() else {
		return false;
	};

	if term_info.match_any("tmux") {
		return set_tmux_pane_title(name);
	} else if term_info.match_any("wezterm") {
		let prog = if let Ok(dir) = env::var("WEZTERM_EXECUTABLE_DIR") {
			Cow::from(format!("{dir}/wezterm"))
		} else {
			Cow::from("wezterm")
		};
		let args = &["cli", "set-tab-title", name];
		let res = crate::support::os::new_run_command(&prog).args(args).spawn();
		return res.is_ok();
	}

	false
}

// endregion: --- Public Functions

// region:    --- Support

fn tmux_pane_title() -> Option<String> {
	let term_info = term_info()?;
	if !term_info.match_any("tmux") {
		return None;
	}

	let output = crate::support::os::new_run_command("tmux")
		.arg("display-message")
		.arg("-p")
		.arg("#T")
		.output()
		.ok()?;

	if !output.status.success() {
		return None;
	}

	let title = String::from_utf8(output.stdout).ok()?;
	let title = title.trim_end_matches(['\r', '\n']).to_string();
	if title.is_empty() {
		Some("term".to_string())
	} else {
		Some(title)
	}
}

fn set_tmux_pane_title(name: &str) -> bool {
	crate::support::os::new_run_command("tmux")
		.arg("select-pane")
		.arg("-T")
		.arg(name)
		.status()
		.is_ok_and(|status| status.success())
}

// endregion: --- Support
