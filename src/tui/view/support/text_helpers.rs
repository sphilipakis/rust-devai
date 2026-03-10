use lazy_regex::regex;

pub struct TextSeg<'a> {
	pub text: String,
	pub file_path: Option<&'a str>,
}

pub fn segment_line_path(line: &str) -> Vec<TextSeg<'_>> {
	// Matches:
	//   - Paths with directories (optionally starting with ~): ~/foo/bar.rs, src/main.rs
	//   - Standalone filenames with extension: tsconfig.json, Cargo.toml
	//   - Dotfiles (with optional chained extensions): .env, .gitignore, .env.local
	let re = regex!(
		r#"~?[a-zA-Z0-9_@\-\./]+/[a-zA-Z0-9_@\-\.]+\.[a-zA-Z0-9]{2,5}|[a-zA-Z0-9_@\-]+\.[a-zA-Z0-9]{2,5}|\.[a-zA-Z0-9_\-]+(?:\.[a-zA-Z0-9]{2,5})*"#
	);
	let mut segments = Vec::new();
	let mut last_idx = 0;

	for m in re.find_iter(line) {
		let start = m.start();
		let end = m.end();
		let text = &line[start..end];

		if start > last_idx {
			segments.push(TextSeg {
				text: line[last_idx..start].to_string(),
				file_path: None,
			});
		}
		segments.push(TextSeg {
			text: text.to_string(),
			file_path: Some(text),
		});
		last_idx = end;
	}

	if last_idx < line.len() {
		segments.push(TextSeg {
			text: line[last_idx..].to_string(),
			file_path: None,
		});
	}

	if segments.is_empty() && !line.is_empty() {
		segments.push(TextSeg {
			text: line.to_string(),
			file_path: None,
		});
	}

	segments
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_text_helpers_segment_line_path_with_slash() -> Result<()> {
		// -- Setup & Fixtures
		let line = "See src/main.rs for details";

		// -- Exec
		let segs = segment_line_path(line);

		// -- Check
		assert_eq!(segs.len(), 3);
		assert_eq!(segs[0].text, "See ");
		assert!(segs[0].file_path.is_none());
		assert_eq!(segs[1].text, "src/main.rs");
		assert_eq!(segs[1].file_path, Some("src/main.rs"));
		assert_eq!(segs[2].text, " for details");
		assert!(segs[2].file_path.is_none());

		Ok(())
	}

	#[test]
	fn test_text_helpers_segment_line_path_tilde_prefix() -> Result<()> {
		// -- Setup & Fixtures
		let line = "Check ~/work/app/src/main.rs now";

		// -- Exec
		let segs = segment_line_path(line);

		// -- Check
		assert_eq!(segs.len(), 3);
		assert_eq!(segs[0].text, "Check ");
		assert_eq!(segs[1].text, "~/work/app/src/main.rs");
		assert_eq!(segs[1].file_path, Some("~/work/app/src/main.rs"));
		assert_eq!(segs[2].text, " now");

		Ok(())
	}

	#[test]
	fn test_text_helpers_segment_line_path_standalone_filename() -> Result<()> {
		// -- Setup & Fixtures
		let line = "Edit tsconfig.json please";

		// -- Exec
		let segs = segment_line_path(line);

		// -- Check
		assert_eq!(segs.len(), 3);
		assert_eq!(segs[0].text, "Edit ");
		assert_eq!(segs[1].text, "tsconfig.json");
		assert_eq!(segs[1].file_path, Some("tsconfig.json"));
		assert_eq!(segs[2].text, " please");

		Ok(())
	}

	#[test]
	fn test_text_helpers_segment_line_path_cargo_toml() -> Result<()> {
		// -- Setup & Fixtures
		let line = "Update Cargo.toml";

		// -- Exec
		let segs = segment_line_path(line);

		// -- Check
		assert_eq!(segs.len(), 2);
		assert_eq!(segs[0].text, "Update ");
		assert_eq!(segs[1].text, "Cargo.toml");
		assert_eq!(segs[1].file_path, Some("Cargo.toml"));

		Ok(())
	}

	#[test]
	fn test_text_helpers_segment_line_path_dotfile_simple() -> Result<()> {
		// -- Setup & Fixtures
		let line = "See .gitignore for exclusions";

		// -- Exec
		let segs = segment_line_path(line);

		// -- Check
		assert_eq!(segs.len(), 3);
		assert_eq!(segs[0].text, "See ");
		assert_eq!(segs[1].text, ".gitignore");
		assert_eq!(segs[1].file_path, Some(".gitignore"));
		assert_eq!(segs[2].text, " for exclusions");

		Ok(())
	}

	#[test]
	fn test_text_helpers_segment_line_path_dotfile_multi_ext() -> Result<()> {
		// -- Setup & Fixtures
		let line = "Check .env.local for overrides";

		// -- Exec
		let segs = segment_line_path(line);

		// -- Check
		assert_eq!(segs.len(), 3);
		assert_eq!(segs[0].text, "Check ");
		assert_eq!(segs[1].text, ".env.local");
		assert_eq!(segs[1].file_path, Some(".env.local"));
		assert_eq!(segs[2].text, " for overrides");

		Ok(())
	}

	#[test]
	fn test_text_helpers_segment_line_path_dotenv() -> Result<()> {
		// -- Setup & Fixtures
		let line = "Load .env vars";

		// -- Exec
		let segs = segment_line_path(line);

		// -- Check
		assert_eq!(segs.len(), 3);
		assert_eq!(segs[0].text, "Load ");
		assert_eq!(segs[1].text, ".env");
		assert_eq!(segs[1].file_path, Some(".env"));
		assert_eq!(segs[2].text, " vars");

		Ok(())
	}

	#[test]
	fn test_text_helpers_segment_line_path_tilde_config() -> Result<()> {
		// -- Setup & Fixtures
		let line = "Edit ~/.config/tool/config.toml";

		// -- Exec
		let segs = segment_line_path(line);

		// -- Check
		assert_eq!(segs.len(), 2);
		assert_eq!(segs[0].text, "Edit ");
		assert_eq!(segs[1].text, "~/.config/tool/config.toml");
		assert_eq!(segs[1].file_path, Some("~/.config/tool/config.toml"));

		Ok(())
	}

	#[test]
	fn test_text_helpers_segment_line_path_no_match() -> Result<()> {
		// -- Setup & Fixtures
		let line = "No files here at all";

		// -- Exec
		let segs = segment_line_path(line);

		// -- Check
		assert_eq!(segs.len(), 1);
		assert_eq!(segs[0].text, "No files here at all");
		assert!(segs[0].file_path.is_none());

		Ok(())
	}
}

// endregion: --- Tests
