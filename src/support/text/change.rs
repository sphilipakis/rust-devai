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
						"Malformed changes: Expected '{LINE_MARKER_SEARCH_START}' or a whitespace line to start a block. Found text outside of a valid block structure. Line: '{}'",
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
			"Malformed change block: Ended in search pattern. Missing separator marker '{LINE_MARKER_SEP}'."
		))),
		ParseState::InReplacePattern { .. } => Err(Error::custom(format!(
			"Malformed change block: Ended in replace pattern. Missing end marker '{LINE_MARKER_REPLACE_END}'."
		))),
	}
}

#[cfg(test)]
#[path = "change_tests.rs"]
mod tests;
