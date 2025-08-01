//! Iterator for extracting marked content sections like <TAG>...</TAG> from text.

/// Represents a segment of text identified by start and end tags,
/// potentially including parameters in the start marker.
///
/// Lifetimes ensure that all string slices (`tag_name`, `attrs_raw`, `content`)
/// are valid references to the original input string slice provided
/// to the `TagIterator`.
#[derive(Debug, PartialEq)]
pub struct TagContent<'a> {
	/// The name of the tag (e.g., "SOME_MARKER").
	pub tag_name: &'a str,

	/// Optional attributes string found within the opening tag.
	/// (e.g., `file_path="some/path/file.txt" other="123"`)
	/// This includes the raw string between the tag name and the closing '>'.
	pub attrs_raw: Option<&'a str>,

	/// The content string between the opening and closing tags.
	pub content: &'a str,

	/// The byte index of the opening '<' of the start tag in the original string.
	pub start_idx: usize,

	/// The byte index of the closing '>' of the end tag in the original string.
	pub end_idx: usize,
}

/// An iterator that finds and extracts `TaggedContent` sections from a string slice.
///
/// It searches for pairs of `<TAG_NAME...>` and `</TAG_NAME>` tags.
pub struct TagContentIterator<'a> {
	input: &'a str,
	tag_name: &'a str,
	current_pos: usize,
	// Precomputed strings for efficiency
	start_tag_prefix: String,
	end_tag: String,
}

impl<'a> TagContentIterator<'a> {
	/// Creates a new `TagIterator` for the given input string and tag name.
	///
	/// # Arguments
	///
	/// * `input` - The string slice to search within.
	/// * `tag_name` - The name of the tag to search for (e.g., "SOME_MARKER").
	#[allow(unused)]
	pub fn new(input: &'a str, tag_name: &'a str) -> Self {
		TagContentIterator {
			input,
			tag_name,
			current_pos: 0,
			start_tag_prefix: format!("<{tag_name}"),
			end_tag: format!("</{tag_name}>"),
		}
	}
}

impl<'a> Iterator for TagContentIterator<'a> {
	type Item = TagContent<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		while self.current_pos < self.input.len() {
			// --- Find the start tag prefix ---
			let remaining_input = &self.input[self.current_pos..];
			let potential_start_offset = remaining_input.find(&self.start_tag_prefix)?;

			let start_idx = self.current_pos + potential_start_offset;
			let after_prefix_idx = start_idx + self.start_tag_prefix.len();

			// --- Validate character after prefix (must be '>' or whitespace) ---
			match self.input.as_bytes().get(after_prefix_idx) {
				Some(b'>') | Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') => {
					// Potential match, proceed
				}
				_ => {
					// It's a different tag (e.g., <TAG_NAMEXXX). Advance past the '<' and continue searching.
					self.current_pos = start_idx + 1;
					continue;
				}
			}

			// --- Find the end of the opening tag '>' ---
			let open_tag_end_offset = match remaining_input[potential_start_offset..].find('>') {
				Some(idx) => potential_start_offset + idx,
				None => {
					// Malformed open tag (no '>'). Stop searching. Consider advancing past '<'?
					// For simplicity, we stop here. A more robust parser might skip.
					return None;
				}
			};
			let open_tag_end_idx = self.current_pos + open_tag_end_offset;

			// --- Extract Parameters ---
			let attrs_raw_str = &self.input[after_prefix_idx..open_tag_end_idx].trim();
			let attrs_raw = if attrs_raw_str.is_empty() {
				None
			} else {
				Some(*attrs_raw_str)
			}; // Make sure to do `*` to have a one &

			// --- Find the closing tag ---
			let search_after_open_tag_idx = open_tag_end_idx + 1;
			if search_after_open_tag_idx >= self.input.len() {
				// Reached end of input before finding closing tag
				return None;
			}

			let remaining_after_open = &self.input[search_after_open_tag_idx..];
			let close_tag_start_offset = remaining_after_open.find(&self.end_tag)?;

			let close_tag_start_idx = search_after_open_tag_idx + close_tag_start_offset;
			// Corrected end_idx calculation: it's the index of the '>' of the closing tag
			// The end index should be the index of the last character of the closing tag '>'
			let end_idx = close_tag_start_idx + self.end_tag.len() - 1;

			// --- Extract Content ---
			let content = &self.input[open_tag_end_idx + 1..close_tag_start_idx];

			// --- Update position for next search ---
			// The next search should start right after the closing tag
			self.current_pos = end_idx + 1;

			// --- Return the found item ---
			return Some(TagContent {
				tag_name: self.tag_name,
				attrs_raw,
				content,
				start_idx,
				end_idx,
			});
		}

		None // Reached end of input
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	// For tests, using a simple Result alias is often sufficient.
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	#[test]
	fn test_support_text_tag_simple() -> Result<()> {
		// -- Setup & Fixtures
		let text = "Some text <DATA>content</DATA> more text.";
		let tag_name = "DATA";

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		assert_eq!(tags.len(), 1);
		assert_eq!(
			tags[0],
			TagContent {
				tag_name: "DATA",
				attrs_raw: None,
				content: "content",
				start_idx: 10,
				end_idx: 29,
			}
		);

		Ok(())
	}

	#[test]
	fn test_support_text_tag_attrs() -> Result<()> {
		// -- Setup & Fixtures
		let text = r#"Before <FILE path="a/b.txt" id=123>File Content</FILE> After"#;
		let tag_name = "FILE";

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		assert_eq!(tags.len(), 1);
		assert_eq!(
			tags[0],
			TagContent {
				tag_name: "FILE",
				attrs_raw: Some(r#"path="a/b.txt" id=123"#),
				content: "File Content",
				start_idx: 7,
				end_idx: 53,
			}
		);

		Ok(())
	}

	#[test]
	fn test_support_text_tag_attrs_with_newline() -> Result<()> {
		// -- Setup & Fixtures
		let text = "Before <FILE \npath=\"a/b.txt\"\n id=123>File Content</FILE> After";
		let tag_name = "FILE";

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		assert_eq!(tags.len(), 1);
		assert_eq!(
			tags[0],
			TagContent {
				tag_name: "FILE",
				attrs_raw: Some("path=\"a/b.txt\"\n id=123"), // Note: .trim() removes leading/trailing whitespace only
				content: "File Content",
				start_idx: 7,
				end_idx: 55,
			}
		);

		Ok(())
	}

	#[test]
	fn test_support_text_tag_multiple() -> Result<()> {
		// -- Setup & Fixtures
		let text = "Data: <ITEM>one</ITEM>, <ITEM key=val>two</ITEM>.";
		let tag_name = "ITEM";

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		assert_eq!(tags.len(), 2);
		assert_eq!(
			tags[0],
			TagContent {
				tag_name: "ITEM",
				attrs_raw: None,
				content: "one",
				start_idx: 6,
				end_idx: 21,
			}
		);
		assert_eq!(
			tags[1],
			TagContent {
				tag_name: "ITEM",
				attrs_raw: Some("key=val"),
				content: "two",
				start_idx: 24, // Corrected from 23 based on error
				end_idx: 47,   // Corrected from 45 based on error
			}
		);

		Ok(())
	}

	#[test]
	fn test_support_text_tag_no_tags() -> Result<()> {
		// -- Setup & Fixtures
		let text = "Just plain text without any tags.";
		let tag_name = "MARKER";

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		assert!(tags.is_empty());

		Ok(())
	}

	#[test]
	fn test_support_text_tag_empty_content() -> Result<()> {
		// -- Setup & Fixtures
		let text = "<EMPTY></EMPTY>";
		let tag_name = "EMPTY";

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		assert_eq!(tags.len(), 1);
		assert_eq!(
			tags[0],
			TagContent {
				tag_name: "EMPTY",
				attrs_raw: None,
				content: "",
				start_idx: 0,
				end_idx: 14,
			}
		);

		Ok(())
	}

	#[test]
	fn test_support_text_tag_nested_like() -> Result<()> {
		// -- Setup & Fixtures
		let text = "<OUTER>outer <INNER>inner</INNER> outer</OUTER>";
		let tag_name_outer = "OUTER";
		let tag_name_inner = "INNER";

		// -- Exec Outer
		let tags_outer: Vec<TagContent> = TagContentIterator::new(text, tag_name_outer).collect();
		// -- Check Outer
		assert_eq!(tags_outer.len(), 1);
		assert_eq!(
			tags_outer[0],
			TagContent {
				tag_name: "OUTER",
				attrs_raw: None,
				content: "outer <INNER>inner</INNER> outer",
				start_idx: 0,
				end_idx: 46,
			}
		);

		// -- Exec Inner
		let tags_inner: Vec<TagContent> = TagContentIterator::new(text, tag_name_inner).collect();
		// -- Check Inner
		assert_eq!(tags_inner.len(), 1);
		assert_eq!(
			tags_inner[0],
			TagContent {
				tag_name: "INNER",
				attrs_raw: None,
				content: "inner",
				start_idx: 13,
				end_idx: 32,
			}
		);

		Ok(())
	}

	#[test]
	fn test_support_text_tag_malformed_open() -> Result<()> {
		// -- Setup & Fixtures
		let text = "<MARKER oops </MARKER>"; // Missing '>'
		let tag_name = "MARKER";

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		// The current implementation stops if '>' isn't found for the opening tag.
		assert!(tags.is_empty());

		Ok(())
	}

	#[test]
	fn test_support_text_tag_unclosed() -> Result<()> {
		// -- Setup & Fixtures
		let text = "<MARKER>content"; // Missing closing tag
		let tag_name = "MARKER";

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		// The current implementation stops if the closing tag isn't found.
		assert!(tags.is_empty());

		Ok(())
	}

	#[test]
	fn test_support_text_tag_edges() -> Result<()> {
		// -- Setup & Fixtures
		let text = "<START>at start</START>middle<END>at end</END>";
		let tag_name_start = "START";
		let tag_name_end = "END";

		// -- Exec Start
		let tags_start: Vec<TagContent> = TagContentIterator::new(text, tag_name_start).collect();
		// -- Check Start
		assert_eq!(tags_start.len(), 1);
		assert_eq!(
			tags_start[0],
			TagContent {
				tag_name: "START",
				attrs_raw: None,
				content: "at start",
				start_idx: 0,
				end_idx: 22,
			}
		);

		// -- Exec End
		let tags_end: Vec<TagContent> = TagContentIterator::new(text, tag_name_end).collect();
		// -- Check End
		assert_eq!(tags_end.len(), 1);
		assert_eq!(
			tags_end[0],
			TagContent {
				tag_name: "END",
				attrs_raw: None,
				content: "at end",
				start_idx: 29,
				end_idx: 45, // Corrected from 46 based on error
			}
		);

		Ok(())
	}

	#[test]
	fn test_support_text_tag_incorrect_tag_name() -> Result<()> {
		// -- Setup & Fixtures
		let text = "<MARKERX>content</MARKERX>";
		let tag_name = "MARKER"; // Searching for MARKER, not MARKERX

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		assert!(tags.is_empty());

		Ok(())
	}

	#[test]
	fn test_support_text_tag_tag_name_prefix_check() -> Result<()> {
		// -- Setup & Fixtures
		let text = "<TAG_EXTRA>extra</TAG_EXTRA><TAG>real</TAG>";
		let tag_name = "TAG";

		// -- Exec
		let tags: Vec<TagContent> = TagContentIterator::new(text, tag_name).collect();

		// -- Check
		assert_eq!(tags.len(), 1);
		assert_eq!(
			tags[0],
			TagContent {
				tag_name: "TAG",
				attrs_raw: None,
				content: "real",
				start_idx: 28,
				end_idx: 42,
			}
		);

		Ok(())
	}
}

// endregion: --- Tests
