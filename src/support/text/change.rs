use crate::support::text::truncate_with_ellipsis;
use crate::{Error, Result};

// These define what a marker line must look like for the parser.
const LINE_MARKER_SEARCH_START: &str = "<<<<<<< SEARCH";
const LINE_MARKER_SEP: &str = "=======";
const LINE_MARKER_REPLACE_END: &str = ">>>>>>> REPLACE";

/// Applies changes to an original content string.
///
/// 1.  **Simple Replacement Mode**: If no line in `changes` exactly matches `<<<<<<< SEARCH`
///     at its beginning, the entire `original_content` is replaced by the `changes` string.
///
/// 2.  **Block Processing Mode**: If `<<<<<<< SEARCH` markers are present (at line starts),
///     `changes` is parsed for blocks:
///     ```
///     <<<<<<< SEARCH
///     search_pattern_line1
///     ...
///     =======
///     replace_pattern_line1
///     ...
///     >>>>>>> REPLACE
///     ```
///     For each block, the *first occurrence* of its `search_pattern` in the (potentially modified)
///     `original_content` is replaced with its `replace_pattern`.
///     Replacements are sequential.
pub fn apply_changes(original_content: &str, changes: impl Into<String>) -> Result<String> {
	let changes_str = changes.into();

	// Determine if we are in block processing mode by checking for any SEARCH_START marker line.
	// TODO: Need to avoid a double scan with below
	let is_block_mode = changes_str.lines().any(|line| line == LINE_MARKER_SEARCH_START);

	if !is_block_mode {
		// Simple Replacement Mode: entire original_content is replaced by changes_str.
		return Ok(changes_str);
	}

	// Block Processing Mode
	let requests = process_change_requests(&changes_str)?;
	let mut current_content = original_content.to_string();

	for req in requests {
		// Extract patterns using indices. process_change_requests ensures valid indices for changes_str.
		let search_pattern = &changes_str[req.search_start_idx..req.search_end_idx];
		let replace_pattern = &changes_str[req.replace_start_idx..req.replace_end_idx];

		// Replace only the first occurrence of search_pattern.
		current_content = current_content.replacen(search_pattern, replace_pattern, 1);
	}

	Ok(current_content)
}

#[derive(Debug)] // For easier debugging if needed during development
struct ChangeRequestIndices {
	search_start_idx: usize,
	search_end_idx: usize, // Exclusive
	replace_start_idx: usize,
	replace_end_idx: usize, // Exclusive
}

/// Parses the `changes_str` to identify all change blocks and returns their byte indices.
/// Markers are only recognized if they are at the beginning of a line.
fn process_change_requests(changes_str: &str) -> Result<Vec<ChangeRequestIndices>> {
	let mut requests = Vec::new();

	enum ParseState {
		ExpectBlockStartOrWhitespace,
		// Stores the byte offset *after* the LINE_MARKER_SEARCH_START line (including its newline),
		// which is the start of the search pattern's content.
		InSearchPattern {
			pattern_start_offset: usize,
		},
		// Stores offsets for the search pattern and the start of the replace pattern's content.
		InReplacePattern {
			search_pattern_start_offset: usize,
			search_pattern_end_offset: usize, // Exclusive
			replace_pattern_start_offset: usize,
		},
	}

	let mut state = ParseState::ExpectBlockStartOrWhitespace;
	let mut current_byte_offset = 0; // Tracks the start of the current line being processed.

	for line_str in changes_str.lines() {
		let line_start_byte_offset = current_byte_offset;

		// Advance offset to the start of the *next* line for the next iteration.
		// This new offset will be used if the current line is a marker,
		// to determine where the *content* of a pattern (search or replace) begins.
		current_byte_offset += line_str.len();
		if current_byte_offset < changes_str.len() {
			// Account for the '\n' stripped by lines() if not the end of the string
			current_byte_offset += 1;
		}

		match state {
			ParseState::ExpectBlockStartOrWhitespace => {
				if line_str.trim().is_empty() {
					// Consume whitespace line, stay in this state
				} else if line_str == LINE_MARKER_SEARCH_START {
					// Content of search pattern starts on the next line.
					// current_byte_offset now points to the start of that next line.
					state = ParseState::InSearchPattern {
						pattern_start_offset: current_byte_offset,
					};
				} else {
					return Err(Error::custom(format!(
						"Malformed changes: Expected '{}' or a whitespace line to start a block. Found text outside of a valid block structure. Line: '{}'",
						LINE_MARKER_SEARCH_START,
						truncate_with_ellipsis(line_str, 100, "...")
					)));
				}
			}
			ParseState::InSearchPattern { pattern_start_offset } => {
				if line_str == LINE_MARKER_SEP {
					// Search pattern content ends just before this separator line.
					let mut search_end = line_start_byte_offset;
					// If the character before this marker line is a newline, exclude it from the pattern.
					if search_end > pattern_start_offset && changes_str.as_bytes().get(search_end - 1) == Some(&b'\n') {
						search_end -= 1;
					}
					// Ensure end index is not less than start (for empty patterns).
					if search_end < pattern_start_offset {
						search_end = pattern_start_offset;
					}

					state = ParseState::InReplacePattern {
						search_pattern_start_offset: pattern_start_offset,
						search_pattern_end_offset: search_end,
						// Replace pattern content starts on the next line (after current_byte_offset).
						replace_pattern_start_offset: current_byte_offset,
					};
				} else {
					// This line is part of the search pattern. Loop continues, indices determined by markers.
				}
			}
			ParseState::InReplacePattern {
				search_pattern_start_offset,
				search_pattern_end_offset,
				replace_pattern_start_offset,
			} => {
				if line_str == LINE_MARKER_REPLACE_END {
					// Replace pattern content ends just before this end marker line.
					let mut replace_end = line_start_byte_offset;
					if replace_end > replace_pattern_start_offset
						&& changes_str.as_bytes().get(replace_end - 1) == Some(&b'\n')
					{
						replace_end -= 1;
					}
					if replace_end < replace_pattern_start_offset {
						replace_end = replace_pattern_start_offset;
					}

					requests.push(ChangeRequestIndices {
						search_start_idx: search_pattern_start_offset,
						search_end_idx: search_pattern_end_offset,
						replace_start_idx: replace_pattern_start_offset,
						replace_end_idx: replace_end,
					});
					state = ParseState::ExpectBlockStartOrWhitespace;
				} else {
					// This line is part of the replace pattern.
				}
			}
		}
	}

	// After iterating all lines, check the final state for unterminated blocks.
	match state {
		ParseState::ExpectBlockStartOrWhitespace => Ok(requests),
		ParseState::InSearchPattern { .. } => Err(Error::custom(format!(
			"Malformed change block: Ended in search pattern. Missing separator marker '{}'.",
			LINE_MARKER_SEP
		))),
		ParseState::InReplacePattern { .. } => Err(Error::custom(format!(
			"Malformed change block: Ended in replace pattern. Missing end marker '{}'.",
			LINE_MARKER_REPLACE_END
		))),
	}
}

// region:    --- Tests
#[cfg(test)]
mod tests {
	use super::*;
	type Result<T = ()> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	// Helper to construct `changes` string for tests, ensuring markers are on their own lines.
	fn format_change_block(search: &str, replace: &str) -> String {
		format!(
			"{}\n{}\n{}\n{}\n{}",
			LINE_MARKER_SEARCH_START, search, LINE_MARKER_SEP, replace, LINE_MARKER_REPLACE_END
		)
	}

	fn format_multiple_change_blocks(blocks: Vec<(&str, &str)>, separator_str: &str) -> String {
		blocks
			.into_iter()
			.map(|(s, r)| format_change_block(s, r))
			.collect::<Vec<String>>()
			.join(separator_str)
	}

	#[test]
	fn test_support_text_apply_change_simple_replace_no_markers() -> Result {
		// -- Setup & Fixtures
		let original = "Hello world";
		let changes = "Hallo Welt".to_string();

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "Hallo Welt");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_simple_replace_contains_end_marker_only_as_text() -> Result {
		// -- Setup & Fixtures
		let original = "Hello world";
		// LINE_MARKER_REPLACE_END is not at the start of a line, so simple mode.
		let changes = format!("Some text {} with an end marker style part", LINE_MARKER_REPLACE_END);

		// -- Exec
		let result = apply_changes(original, changes.clone())?;

		// -- Check
		assert_eq!(result, changes);
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_single_valid_block_original_markers_format() -> Result {
		// -- Setup & Fixtures
		let original = "Hello old_text world, and old_text again.";
		// The old MARKER_... constants included newlines.
		// The format_change_block helper ensures clean lines now.
		let changes = format_change_block("old_text", "new_text");

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		// replacen replaces only the first "old_text".
		assert_eq!(result, "Hello new_text world, and old_text again.");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_single_valid_block_explicit_lines() -> Result {
		// -- Setup & Fixtures
		let original = "Hello old_text1 world.";
		let changes = format_change_block("old_text1", "new_text1");

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "Hello new_text1 world.");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_multiline_patterns_in_block_new_format() -> Result {
		// -- Setup & Fixtures
		let original = "First line\nSecond old line\nThird line";
		let search_pattern_text = "Second old line";
		let replace_pattern_text = "Second new line\nAnd a new third line";

		let changes = format_change_block(search_pattern_text, replace_pattern_text);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		let expected = "First line\nSecond new line\nAnd a new third line\nThird line";
		assert_eq!(result, expected);
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_multiple_valid_blocks() -> Result {
		// -- Setup & Fixtures
		let original = "one two three two one";
		let blocks_data = vec![("one", "1"), ("two", "2"), ("three", "3")];
		let changes = format_multiple_change_blocks(blocks_data, "\n");

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		// 1. "one two three two one".replacen("one", "1", 1) -> "1 two three two one"
		// 2. "1 two three two one".replacen("two", "2", 1) -> "1 2 three two one"
		// 3. "1 2 three two one".replacen("three", "3", 1) -> "1 2 3 two one"
		assert_eq!(result, "1 2 3 two one");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_multiple_blocks_with_newlines_between() -> Result {
		// -- Setup & Fixtures
		let original = "one two three two one";
		let blocks_data = vec![("one", "1"), ("two", "2"), ("three", "3")];
		let changes = format_multiple_change_blocks(blocks_data, "\n\n\n"); // Whitespace lines between blocks

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "1 2 3 two one"); // Same logic as above
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_multiple_blocks_with_mixed_whitespace_and_trailing_clean_markers() -> Result {
		// -- Setup & Fixtures
		let original = "apple banana cherry";
		let block1 = format_change_block("apple", "A");
		let block2 = format_change_block("banana", "B");
		let block3 = format_change_block("cherry", "C");
		// TODO: Should allow spaced after the markers on same line
		let changes = format!("{}\n{}\n{}", block1, block2, block3);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		// 1. "apple banana cherry".replacen("apple", "A", 1) -> "A banana cherry"
		// 2. "A banana cherry".replacen("banana", "B", 1) -> "A B cherry"
		// 3. "A B cherry".replacen("cherry", "C", 1) -> "A B C"
		assert_eq!(result, "A B C");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_error_marker_line_with_trailing_whitespace() -> Result {
		// -- Setup & Fixtures
		let original = "apple";
		let changes = format!(
			"{}\napple\n{}\n{}\n{}",
			LINE_MARKER_SEARCH_START,
			LINE_MARKER_SEP,
			"A",
			LINE_MARKER_REPLACE_END // This is correct
		)
		// Now make one of the marker lines incorrect by adding trailing space
		.replace(LINE_MARKER_SEP, &format!("{} ", LINE_MARKER_SEP)); // "======= "

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to malformed marker line");
		if let Err(e) = result {
			// The line "======= " is not a valid separator, so it's part of search pattern.
			// Parsing ends in InSearchPattern state.
			assert!(
				e.to_string()
					.contains(&format!("Missing separator marker '{}'", LINE_MARKER_SEP)),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_error_with_preamble() -> Result {
		// -- Setup & Fixtures
		let original = "Replace target.";
		let changes = format!(
			"Some preamble text, should cause error.\n{}\ntarget\n{}\nreplacement\n{}",
			LINE_MARKER_SEARCH_START, LINE_MARKER_SEP, LINE_MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to preamble text");
		if let Err(e) = result {
			assert!(
				e.to_string().contains("Expected '<<<<<<< SEARCH' or a whitespace line"),
				"Error message mismatch: {}",
				e
			);
			assert!(
				e.to_string().contains("Line: 'Some preamble text"),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_error_with_interstitial_text() -> Result {
		// -- Setup & Fixtures
		let original = "one two";
		let block1 = format_change_block("one", "1");
		let block2 = format_change_block("two", "2");
		let changes = format!("{}\n  Some interstitial text, should cause error.\n{}", block1, block2);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to interstitial text");
		if let Err(e) = result {
			assert!(
				e.to_string().contains("Expected '<<<<<<< SEARCH' or a whitespace line"),
				"Error message mismatch: {}",
				e
			);
			assert!(
				e.to_string().contains("Line: '  Some interstitial text"),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_error_with_trailing_text() -> Result {
		// -- Setup & Fixtures
		let original = "Replace target.";
		let block = format_change_block("target", "replacement");
		let changes = format!("{}\n  Some trailing text, should cause error.", block);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to trailing text");
		if let Err(e) = result {
			assert!(
				e.to_string().contains("Expected '<<<<<<< SEARCH' or a whitespace line"),
				"Error message mismatch: {}",
				e
			);
			assert!(
				e.to_string().contains("Line: '  Some trailing text"),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_search_not_found_in_original() -> Result {
		// -- Setup & Fixtures
		let original = "Hello world";
		let changes = format_change_block("not_found_text", "replacement");

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(
			result, original,
			"Original should be unchanged as search pattern not found"
		);
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_empty_search_pattern() -> Result {
		// -- Setup & Fixtures
		let original = "abc";
		let changes = format_change_block("", "X"); // Empty search pattern

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		// "abc".replacen("", "X", 1) results in "Xabc"
		assert_eq!(result, "Xabc");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_empty_replace_pattern() -> Result {
		// -- Setup & Fixtures
		let original = "delete this text now";
		let changes = format_change_block("this text", ""); // Empty replace pattern

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "delete  now");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_malformed_missing_separator() -> Result {
		// -- Setup & Fixtures
		let original = "Hello world";
		let changes = format!(
			"{}\nsearch_text_no_sep_then_end\n{}", // Missing LINE_MARKER_SEP
			LINE_MARKER_SEARCH_START, LINE_MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to missing separator");
		if let Err(e) = result {
			assert!(
				e.to_string()
					.contains(&format!("Missing separator marker '{}'", LINE_MARKER_SEP)),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_malformed_missing_end() -> Result {
		// -- Setup & Fixtures
		let original = "Hello world";
		let changes = format!(
			"{}\nsearch_text\n{}\nreplace_text_no_end", // Missing LINE_MARKER_REPLACE_END
			LINE_MARKER_SEARCH_START, LINE_MARKER_SEP
		);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to missing end marker");
		if let Err(e) = result {
			assert!(
				e.to_string()
					.contains(&format!("Missing end marker '{}'", LINE_MARKER_REPLACE_END)),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_markers_as_text_in_patterns() -> Result {
		// -- Setup & Fixtures
		let original = format!(
			"Content with {} inside, and also {} and {}.",
			LINE_MARKER_SEARCH_START, LINE_MARKER_SEP, LINE_MARKER_REPLACE_END
		);
		// Search for the literal text of a marker, not as a structural marker.
		let search_pattern = LINE_MARKER_SEARCH_START; // e.g. "<<<<<<< SEARCH"
		let replace_pattern = "FOUND_IT";
		let changes = format_change_block(search_pattern, replace_pattern);

		// -- Exec
		let result = apply_changes(original.as_str(), changes)?;

		// -- Check
		assert_eq!(
			result,
			format!(
				"Content with FOUND_IT inside, and also {} and {}.", // Only first "<<<<<<< SEARCH" is replaced
				LINE_MARKER_SEP, LINE_MARKER_REPLACE_END
			)
		);
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_block_at_eof_no_trailing_newline_in_changes_input() -> Result {
		// -- Setup & Fixtures
		let original = "fix this";
		// format_change_block ensures markers are on their own lines.
		// The `changes_str` itself might not have a final newline after the last marker line.
		// This is handled fine by `lines()` iterator.
		let changes_str_no_final_newline = format_change_block("this", "that");

		// -- Exec
		let result = apply_changes(original, changes_str_no_final_newline)?;

		// -- Check
		assert_eq!(result, "fix that");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_empty_changes_string() -> Result {
		// -- Setup & Fixtures
		let original = "original content";
		let changes = "".to_string();

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		// Empty changes string -> simple replace mode -> replaces original with ""
		assert_eq!(result, "");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_changes_is_just_search_start_marker_line() -> Result {
		// -- Setup & Fixtures
		let original = "original";
		let changes = LINE_MARKER_SEARCH_START.to_string(); // Just one line: "<<<<<<< SEARCH"

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err());
		if let Err(e) = result {
			// Ends in search pattern, missing separator
			assert!(
				e.to_string()
					.contains(&format!("Missing separator marker '{}'", LINE_MARKER_SEP))
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_changes_is_incomplete_block_missing_replace_text_and_end_marker() -> Result {
		// -- Setup & Fixtures
		let original = "original";
		let changes = format!("{}\nsearch_text\n{}", LINE_MARKER_SEARCH_START, LINE_MARKER_SEP);
		// Lines: "<<<<<<< SEARCH", "search_text", "======="

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err());
		if let Err(e) = result {
			// Ends in replace pattern (empty), missing end marker
			assert!(
				e.to_string()
					.contains(&format!("Missing end marker '{}'", LINE_MARKER_REPLACE_END))
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_empty_search_pattern_in_block() -> Result {
		// -- Setup & Fixtures
		let original = "abc def";
		let changes = format_change_block("", "X-"); // search="", replace="X-"

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "X-abc def");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_empty_replace_pattern_in_block() -> Result {
		// -- Setup & Fixtures
		let original = "abc remove_this def";
		let changes = format_change_block("remove_this", ""); // search="remove_this", replace=""

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "abc  def");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_multiline_search_and_replace() -> Result {
		// -- Setup & Fixtures
		let original = "line one\nline two\nline three\nline four";
		let search = "line two\nline three";
		let replace = "new line A\nnew line B";
		let changes = format_change_block(search, replace);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "line one\nnew line A\nnew line B\nline four");
		Ok(())
	}
}
// endregion: --- Tests
