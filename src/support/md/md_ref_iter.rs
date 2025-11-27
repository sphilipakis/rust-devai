use crate::support::md::InBlockState;
use crate::types::{MdRef, MdRefKind};
use lazy_regex::regex;

/// Represents an iterator over Markdown links and references.
pub struct MdRefIter<'a> {
	//content: &'a str,
	/// Current position in the content
	//pos: usize,
	/// Track code block state to skip content inside code blocks
	block_state: InBlockState,
	/// Lines iterator for block state tracking
	lines: std::iter::Peekable<std::str::Lines<'a>>,
	/// Current line being processed
	current_line: Option<&'a str>,
	/// Position within the current line
	line_pos: usize,
	/// Absolute position of the start of current line
	line_start: usize,
}

impl<'a> MdRefIter<'a> {
	/// Creates a new MdRefIter from the given content.
	pub fn new(content: &'a str) -> Self {
		let mut lines = content.lines().peekable();
		let current_line = lines.next();
		MdRefIter {
			//content,
			//pos: 0,
			block_state: InBlockState::Out,
			lines,
			current_line,
			line_pos: 0,
			line_start: 0,
		}
	}

	/// Advance to the next line
	fn advance_line(&mut self) {
		if let Some(current) = self.current_line {
			self.line_start += current.len() + 1; // +1 for newline
			self.current_line = self.lines.next();
			self.line_pos = 0;
		}
	}

	/// Find the next reference in the content
	fn next_ref(&mut self) -> Option<MdRef> {
		// Pattern to match markdown links: ![text](url) or [text](url)
		// We'll process character by character to handle code blocks properly
		let re = regex!(r"(!?\[)([^\]]*)\]\(([^)]+)\)");

		while let Some(line) = self.current_line {
			// Update block state for the current line
			let new_state = self.block_state.compute_new(line);

			// If we're entering or inside a code block, skip this line
			if !new_state.is_out() {
				self.block_state = new_state;
				self.advance_line();
				continue;
			}

			// If we just exited a code block, update state and continue
			if !self.block_state.is_out() && new_state.is_out() {
				self.block_state = new_state;
				self.advance_line();
				continue;
			}

			self.block_state = new_state;

			// Search for references in the remaining part of the line
			let search_start = self.line_pos;
			let line_remainder = &line[search_start..];

			// Check if we're inside inline code (single backtick)
			if let Some(cap) = re.captures(line_remainder) {
				let match_start = cap.get(0).unwrap().start();
				let match_end = cap.get(0).unwrap().end();

				// Check if this match is inside inline code
				let prefix = &line[..search_start + match_start];
				let backtick_count = prefix.chars().filter(|&c| c == '`').count();

				// If odd number of backticks before, we're inside inline code
				if backtick_count % 2 == 1 {
					self.line_pos = search_start + match_end;
					continue;
				}

				let bracket = cap.get(1).unwrap().as_str();
				let text = cap.get(2).unwrap().as_str();
				let target = cap.get(3).unwrap().as_str();

				let inline = bracket == "![";
				let text = if text.is_empty() { None } else { Some(text.to_string()) };
				let kind = MdRefKind::from_target(target);

				// Update position for next search
				self.line_pos = search_start + match_end;

				return Some(MdRef {
					target: target.to_string(),
					text,
					inline,
					kind,
				});
			}

			// No more matches on this line, move to next
			self.advance_line();
		}

		None
	}
}

impl Iterator for MdRefIter<'_> {
	type Item = MdRef;

	fn next(&mut self) -> Option<Self::Item> {
		self.next_ref()
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_md_ref_iter_simple_link() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "[click here](https://example.com)";

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 1);
		let md_ref = &refs[0];
		assert_eq!(md_ref.target, "https://example.com");
		assert_eq!(md_ref.text.as_deref(), Some("click here"));
		assert!(!md_ref.inline);
		assert_eq!(md_ref.kind, MdRefKind::Url);

		Ok(())
	}

	#[test]
	fn test_md_ref_iter_image() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "![alt text](image.png)";

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 1);
		let md_ref = &refs[0];
		assert_eq!(md_ref.target, "image.png");
		assert_eq!(md_ref.text.as_deref(), Some("alt text"));
		assert!(md_ref.inline);
		assert_eq!(md_ref.kind, MdRefKind::File);

		Ok(())
	}

	#[test]
	fn test_md_ref_iter_anchor() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "[go to section](#my-section)";

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 1);
		let md_ref = &refs[0];
		assert_eq!(md_ref.target, "#my-section");
		assert_eq!(md_ref.text.as_deref(), Some("go to section"));
		assert!(!md_ref.inline);
		assert_eq!(md_ref.kind, MdRefKind::Anchor);

		Ok(())
	}

	#[test]
	fn test_md_ref_iter_multiple_links() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = r#"
Check out [this link](https://example.com) and [another](docs/page.md).

Also see ![image](assets/photo.jpg) for reference.
"#;

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 3);

		assert_eq!(refs[0].target, "https://example.com");
		assert_eq!(refs[0].kind, MdRefKind::Url);
		assert!(!refs[0].inline);

		assert_eq!(refs[1].target, "docs/page.md");
		assert_eq!(refs[1].kind, MdRefKind::File);
		assert!(!refs[1].inline);

		assert_eq!(refs[2].target, "assets/photo.jpg");
		assert_eq!(refs[2].kind, MdRefKind::File);
		assert!(refs[2].inline);

		Ok(())
	}

	#[test]
	fn test_md_ref_iter_skip_code_block() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = r#"
Here is a [real link](https://real.com).

```
[not a link](https://fake.com)
```

And [another real](page.md).
"#;

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 2);
		assert_eq!(refs[0].target, "https://real.com");
		assert_eq!(refs[1].target, "page.md");

		Ok(())
	}

	#[test]
	fn test_md_ref_iter_skip_code_block_4_backticks() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = r#"
Here is a [real link](https://real.com).

````
[not a link](https://fake.com)
````

And [another real](page.md).
"#;

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 2);
		assert_eq!(refs[0].target, "https://real.com");
		assert_eq!(refs[1].target, "page.md");

		Ok(())
	}

	#[test]
	fn test_md_ref_iter_skip_inline_code() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = r#"
Here is a [real link](https://real.com).

This is `[not a link](https://fake.com)` inline code.

And [another real](page.md).
"#;

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 2);
		assert_eq!(refs[0].target, "https://real.com");
		assert_eq!(refs[1].target, "page.md");

		Ok(())
	}

	#[test]
	fn test_md_ref_iter_empty_text() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "[](https://example.com)";

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 1);
		let md_ref = &refs[0];
		assert_eq!(md_ref.target, "https://example.com");
		assert!(md_ref.text.is_none());

		Ok(())
	}

	#[test]
	fn test_md_ref_iter_protocol_relative_url() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "[link](//example.com/path)";

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 1);
		assert_eq!(refs[0].kind, MdRefKind::Url);

		Ok(())
	}

	#[test]
	fn test_md_ref_iter_multiple_on_same_line() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "[first](a.md) and [second](b.md) and [third](c.md)";

		// -- Exec
		let refs: Vec<MdRef> = MdRefIter::new(fx_content).collect();

		// -- Check
		assert_eq!(refs.len(), 3);
		assert_eq!(refs[0].target, "a.md");
		assert_eq!(refs[1].target, "b.md");
		assert_eq!(refs[2].target, "c.md");

		Ok(())
	}
}

// endregion: --- Tests
