//! Iterator for extracting marked content sections like <TAG>...</TAG> from text.

// region:    --- Types

/// Precomputed tag information for iteration efficiency.
pub struct TagInfo {
	pub name: String,
	pub start_tag_prefix: String,
	pub end_tag: String,
}

impl TagInfo {
	pub fn new(tag_name: &str) -> Self {
		TagInfo {
			name: tag_name.to_string(),
			start_tag_prefix: format!("<{tag_name}"),
			end_tag: format!("</{tag_name}>"),
		}
	}
}

// endregion: --- Types

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
	current_pos: usize,
	tag_infos: Vec<TagInfo>,
}

impl<'a> TagContentIterator<'a> {
	/// Creates a new `TagIterator` for the given input string and tag name.
	///
	/// # Arguments
	///
	/// * `input` - The string slice to search within.
	/// * `tag_name` - The name of the tag to search for (e.g., "SOME_MARKER").
	#[allow(unused)]
	pub fn new(input: &'a str, tag_names: &[&str]) -> Self {
		let tag_infos = tag_names.iter().map(|&name| TagInfo::new(name)).collect();
		TagContentIterator {
			input,
			current_pos: 0,
			tag_infos,
		}
	}
}

impl<'a> Iterator for TagContentIterator<'a> {
	type Item = TagContent<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		// TODO: Need to make it work on muli tag_infos.
		while self.current_pos < self.input.len() {
			// --- Find the start tag prefix ---
			let remaining_input = &self.input[self.current_pos..];
			let mut selected: Option<(usize, &TagInfo)> = None;

			for tag_info in &self.tag_infos {
				if let Some(offset) = remaining_input.find(&tag_info.start_tag_prefix) {
					let start_idx = self.current_pos + offset;

					selected = match selected {
						None => Some((start_idx, tag_info)),
						Some((existing_idx, existing_tag)) => {
							if start_idx < existing_idx
								|| (start_idx == existing_idx && tag_info.name.len() > existing_tag.name.len())
							{
								Some((start_idx, tag_info))
							} else {
								Some((existing_idx, existing_tag))
							}
						}
					};
				}
			}

			let (start_idx, tag_info) = selected?;

			let after_prefix_idx = start_idx + tag_info.start_tag_prefix.len();

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
			let remaining_from_start = &self.input[start_idx..];
			let open_tag_end_offset = match remaining_from_start.find('>') {
				Some(idx) => idx,
				None => {
					// Malformed open tag (no '>'). Stop searching. Consider advancing past '<'?
					// For simplicity, we stop here. A more robust parser might skip.
					return None;
				}
			};
			let open_tag_end_idx = start_idx + open_tag_end_offset;

			// Note: Tag name is derived from input slice using tag_info.name length.
			// Tag name starts at start_idx + 1
			let tag_name_len = tag_info.name.len();
			let tag_name = &self.input[start_idx + 1..start_idx + 1 + tag_name_len];

			// --- Extract Parameters ---
			let attrs_section = &self.input[after_prefix_idx..open_tag_end_idx];
			let attrs_raw_str = attrs_section.trim();
			let attrs_raw = if attrs_raw_str.is_empty() {
				None
			} else {
				Some(attrs_raw_str)
			}; // Keep attrs referencing the original input slice.

			// --- Find the closing tag ---
			let search_after_open_tag_idx = open_tag_end_idx + 1;
			if search_after_open_tag_idx >= self.input.len() {
				// Reached end of input before finding closing tag
				return None;
			}

			let remaining_after_open = &self.input[search_after_open_tag_idx..];
			let close_tag_start_offset = remaining_after_open.find(&tag_info.end_tag)?;
			let close_tag_start_idx = search_after_open_tag_idx + close_tag_start_offset;
			// Corrected end_idx calculation: it's the index of the '>' of the closing tag
			// The end index should be the index of the last character of the closing tag '>'
			let end_idx = close_tag_start_idx + tag_info.end_tag.len() - 1;

			// --- Extract Content ---
			let content = &self.input[open_tag_end_idx + 1..close_tag_start_idx];

			// --- Update position for next search ---
			// The next search should start right after the closing tag
			self.current_pos = end_idx + 1;

			// --- Return the found item ---
			return Some(TagContent {
				tag_name,
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

#[path = "tag_content_iter_tests.rs"]
#[cfg(test)]
mod tests;

// endregion: --- Tests
