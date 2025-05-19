use crate::support::text::truncate_with_ellipsis;
use crate::{Error, Result};

const MARKER_SEARCH_START: &str = "<<<<<<< SEARCH\n";
const MARKER_SEARCH_REPLACE_SEP: &str = "\n=======\n";
const MARKER_REPLACE_END: &str = "\n>>>>>>> REPLACE";

/// Applies changes to an original content string.
///
/// The `changes` string can be one of two forms:
/// 1. Simple Replacement: If `changes` does not contain `<<<<<<< SEARCH`, the entire
///    `original_content` is replaced by the `changes` string itself.
/// 2. SEARCH/REPLACE Blocks: If `changes` contains `<<<<<<< SEARCH`, it must consist
///    entirely of one or more blocks of the format:
///    ```
///    <<<<<<< SEARCH
///    text_to_find
///    =======
///    replacement_text
///    >>>>>>> REPLACE
///    ```
///    (Note: the markers include specific newline characters as defined by constants.)
///    No other text is allowed before, between, or after these blocks.
///    Each `text_to_find` in the `original_content` is replaced with its
///    corresponding `replacement_text`. Replacements are sequential, using the
///    output of the previous replacement as input for the next.
pub fn apply_changes(original_content: &str, changes: impl Into<String>) -> Result<String> {
	let changes_str = changes.into();

	// If MARKER_SEARCH_START is not present, it's a simple replacement.
	if !changes_str.contains(MARKER_SEARCH_START) {
		return Ok(changes_str);
	}

	// If MARKER_SEARCH_START is present, the entire string must conform to the block structure.
	let mut current_content = original_content.to_string();
	let mut offset = 0;

	while offset < changes_str.len() {
		// 1. Expect MARKER_SEARCH_START at the current offset
		if !changes_str[offset..].starts_with(MARKER_SEARCH_START) {
			return Err(Error::custom(format!(
				"Malformed changes: Expected '{}' at offset {}. Found text outside of a valid block structure. Context: '{}'",
				MARKER_SEARCH_START.trim_end(), // For display
				offset,
				truncate_with_ellipsis(&changes_str[offset..], 100, "...")
			)));
		}
		let search_pattern_start_abs_pos = offset + MARKER_SEARCH_START.len();

		// 2. Find MARKER_SEARCH_REPLACE_SEP
		// If changes_str == MARKER_SEARCH_START, then changes_str[search_pattern_start_abs_pos..] is empty.
		// `find` on an empty string will return None, leading to the "missing separator" error in the `else` block below.
		if let Some(sep_marker_rel_pos) = changes_str[search_pattern_start_abs_pos..].find(MARKER_SEARCH_REPLACE_SEP) {
			let sep_marker_abs_pos = search_pattern_start_abs_pos + sep_marker_rel_pos;
			let search_pattern = &changes_str[search_pattern_start_abs_pos..sep_marker_abs_pos];

			let replace_pattern_start_abs_pos = sep_marker_abs_pos + MARKER_SEARCH_REPLACE_SEP.len();

			// 3. Find MARKER_REPLACE_END and extract replacement_text
			// At this point, MARKER_SEARCH_START and MARKER_SEARCH_REPLACE_SEP are known to be valid and fully contained in changes_str.
			// replace_pattern_start_abs_pos points to where the replacement text (if any) should begin.
			// Standard string `find` ensures that if it returns Some, the full MARKER_SEARCH_REPLACE_SEP was found.
			// Thus, replace_pattern_start_abs_pos will be <= changes_str.len().

			if replace_pattern_start_abs_pos == changes_str.len() {
				// String ends *exactly* after the separator. No room for replacement text or end marker.
				return Err(Error::custom(format!(
					"Malformed change block: Block started with '{}' at offset {} and had separator '{}', but ended prematurely before replacement text or end marker.",
					MARKER_SEARCH_START.trim_end(),
					offset,
					MARKER_SEARCH_REPLACE_SEP.trim(),
				)));
			} else {
				// replace_pattern_start_abs_pos < changes_str.len()
				// There is some text after the separator. Try to find END marker in it.
				if let Some(end_marker_rel_pos) = changes_str[replace_pattern_start_abs_pos..].find(MARKER_REPLACE_END)
				{
					let end_marker_abs_pos = replace_pattern_start_abs_pos + end_marker_rel_pos;
					let replace_pattern = &changes_str[replace_pattern_start_abs_pos..end_marker_abs_pos];

					current_content = current_content.replace(search_pattern, replace_pattern);

					offset = end_marker_abs_pos + MARKER_REPLACE_END.len();
				} else {
					// Found text after separator, but no END marker.
					return Err(Error::custom(format!(
						"Malformed change block: missing end marker '{}' after pattern starting at offset {}. Block context: '{}'",
						MARKER_REPLACE_END.trim_start(),
						offset,
						truncate_with_ellipsis(&changes_str[offset..], 100, "...")
					)));
				}
			}
		} else {
			// MARKER_SEARCH_REPLACE_SEP not found
			return Err(Error::custom(format!(
				"Malformed change block: missing separator marker '{}' after search block starting at offset {}. Block context: '{}'",
				MARKER_SEARCH_REPLACE_SEP.trim(), // For display
				offset,                           // start of current <<<<<<< SEARCH block
				truncate_with_ellipsis(&changes_str[offset..], 100, "...")
			)));
		}
	}

	Ok(current_content)
}

// region:    --- Tests
#[cfg(test)]
mod tests {
	use super::*;
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	#[test]
	fn test_support_text_apply_change_simple_replace_no_markers() -> Result<()> {
		// -- Setup & Fixtures
		let original = "Hello world";
		let changes = "Hallo Welt".to_string(); // Does not contain "<<<<<<< SEARCH"

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "Hallo Welt");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_simple_replace_contains_end_marker_only() -> Result<()> {
		// -- Setup & Fixtures
		let original = "Hello world";
		// Contains ">>>>>>> REPLACE" but not "<<<<<<< SEARCH"
		let changes = format!(
			"Some text {} with an end marker style part",
			MARKER_REPLACE_END.trim_start()
		);

		// -- Exec
		let result = apply_changes(original, changes.clone())?;

		// -- Check
		// Should be treated as simple replacement because MARKER_SEARCH_START is missing
		assert_eq!(result, changes);
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_single_valid_block() -> Result<()> {
		// -- Setup & Fixtures
		let original = "Hello old_text world, and old_text again.";
		let changes = format!(
			"{}old_text{}new_text{}",
			MARKER_SEARCH_START, MARKER_SEARCH_REPLACE_SEP, MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "Hello new_text world, and new_text again.");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_multiple_valid_blocks() -> Result<()> {
		// -- Setup & Fixtures
		let original = "one two three two one";
		// Blocks are contiguous
		let changes = format!(
			"{search_start}one{sep}1{end}{search_start}two{sep}2{end}{search_start}three{sep}3{end}",
			search_start = MARKER_SEARCH_START,
			sep = MARKER_SEARCH_REPLACE_SEP,
			end = MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "1 2 3 2 1");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_error_with_preamble() -> Result<()> {
		// -- Setup & Fixtures
		let original = "Replace target.";
		let changes = format!(
			"Some preamble text, should cause error.\n{search_start}target{sep}replacement{end}",
			search_start = MARKER_SEARCH_START,
			sep = MARKER_SEARCH_REPLACE_SEP,
			end = MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to preamble text");
		if let Err(e) = result {
			assert!(
				e.to_string().contains("Expected '<<<<<<< SEARCH' at offset 0"),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_error_with_interstitial_text() -> Result<()> {
		// -- Setup & Fixtures
		let original = "one two";
		let changes = format!(
			"{search_start}one{sep}1{end}\nSome interstitial text, should cause error.\n{search_start}two{sep}2{end}",
			search_start = MARKER_SEARCH_START,
			sep = MARKER_SEARCH_REPLACE_SEP,
			end = MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to interstitial text");
		if let Err(e) = result {
			let first_block_len = MARKER_SEARCH_START.len()
				+ "one".len()
				+ MARKER_SEARCH_REPLACE_SEP.len()
				+ "1".len() + MARKER_REPLACE_END.len();
			assert!(
				e.to_string()
					.contains(&format!("Expected '<<<<<<< SEARCH' at offset {}", first_block_len)),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_error_with_trailing_text() -> Result<()> {
		// -- Setup & Fixtures
		let original = "Replace target.";
		let changes = format!(
			"{search_start}target{sep}replacement{end}\nSome trailing text, should cause error.",
			search_start = MARKER_SEARCH_START,
			sep = MARKER_SEARCH_REPLACE_SEP,
			end = MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to trailing text");
		if let Err(e) = result {
			let first_block_len = MARKER_SEARCH_START.len()
				+ "target".len()
				+ MARKER_SEARCH_REPLACE_SEP.len()
				+ "replacement".len()
				+ MARKER_REPLACE_END.len();
			assert!(
				e.to_string()
					.contains(&format!("Expected '<<<<<<< SEARCH' at offset {}", first_block_len)),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_search_not_found_in_original() -> Result<()> {
		// -- Setup & Fixtures
		let original = "Hello world";
		let changes = format!(
			"{}not_found_text{}replacement{}",
			MARKER_SEARCH_START, MARKER_SEARCH_REPLACE_SEP, MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(
			result, original,
			"Original should be unchanged if search pattern not found"
		);
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_empty_search_pattern() -> Result<()> {
		// -- Setup & Fixtures
		let original = "abc";
		let changes = format!(
			"{}{}{}{}",
			MARKER_SEARCH_START, MARKER_SEARCH_REPLACE_SEP, "X", MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		let expected = "X".to_string() + &original.chars().map(|c| format!("{}X", c)).collect::<String>();
		assert_eq!(result, expected);
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_empty_replace_pattern() -> Result<()> {
		// -- Setup & Fixtures
		let original = "delete this text now";
		let changes = format!(
			"{}this text{}{}{}",
			MARKER_SEARCH_START, MARKER_SEARCH_REPLACE_SEP, "", MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "delete  now");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_malformed_missing_separator() -> Result<()> {
		// -- Setup & Fixtures
		let original = "Hello world";
		let changes = format!(
			"{}search_text_no_sep_then_end{}",
			MARKER_SEARCH_START, MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to missing separator");
		if let Err(e) = result {
			assert!(
				e.to_string().contains("missing separator marker '======='"),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_malformed_missing_end() -> Result<()> {
		// -- Setup & Fixtures
		let original = "Hello world";
		let changes = format!(
			"{}search_text{}replace_text_no_end",
			MARKER_SEARCH_START, MARKER_SEARCH_REPLACE_SEP
		);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err(), "Should fail due to missing end marker");
		if let Err(e) = result {
			assert!(
				e.to_string().contains("missing end marker '>>>>>>> REPLACE'"),
				"Error message mismatch: {}",
				e
			);
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_multiline_patterns_in_block() -> Result<()> {
		// -- Setup & Fixtures
		let original = "First line\nSecond old line\nThird line";
		let search_pattern = "Second old line";
		let replace_pattern = "Second new line\nAnd a new third line";
		let changes = format!(
			"{}{}{}{}{}",
			MARKER_SEARCH_START, search_pattern, MARKER_SEARCH_REPLACE_SEP, replace_pattern, MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		let expected = "First line\nSecond new line\nAnd a new third line\nThird line";
		assert_eq!(result, expected);
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_markers_as_text_in_patterns() -> Result<()> {
		// -- Setup & Fixtures
		let original = "Content with <<<<<<< SEARCH inside, and also ======= and >>>>>>> REPLACE.";
		let search_pattern = "<<<<<<< SEARCH"; // The text to find, not the marker itself
		let replace_pattern = "FOUND_IT";
		let changes = format!(
			"{search_start}{pattern_as_text}{sep}{replacement}{end}",
			search_start = MARKER_SEARCH_START,
			pattern_as_text = search_pattern,
			sep = MARKER_SEARCH_REPLACE_SEP,
			replacement = replace_pattern,
			end = MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(
			result,
			"Content with FOUND_IT inside, and also ======= and >>>>>>> REPLACE."
		);
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_block_at_eof_no_trailing_newline_in_changes() -> Result<()> {
		// -- Setup & Fixtures
		let original = "fix this";
		// MARKER_REPLACE_END is "\n>>>>>>> REPLACE", so changes_str will end with "REPLACE"
		let changes_no_trailing_newline_after_marker = format!(
			"{}this{}that{}",
			MARKER_SEARCH_START, MARKER_SEARCH_REPLACE_SEP, MARKER_REPLACE_END
		);

		// -- Exec
		let result = apply_changes(original, changes_no_trailing_newline_after_marker)?;

		// -- Check
		assert_eq!(result, "fix that");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_empty_changes_string() -> Result<()> {
		// -- Setup & Fixtures
		let original = "original content";
		let changes = "".to_string();

		// -- Exec
		let result = apply_changes(original, changes)?;

		// -- Check
		assert_eq!(result, "");
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_changes_is_just_search_start_marker() -> Result<()> {
		// -- Setup & Fixtures
		let original = "original";
		let changes = MARKER_SEARCH_START.to_string();

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err());
		if let Err(e) = result {
			assert!(e.to_string().contains("missing separator marker '======='"));
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_changes_is_incomplete_block_missing_replace_text_and_end_marker() -> Result<()> {
		// -- Setup & Fixtures
		let original = "original";
		let changes = format!("{}search_text{}", MARKER_SEARCH_START, MARKER_SEARCH_REPLACE_SEP);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err());
		if let Err(e) = result {
			assert!(e.to_string().contains(
				"Block started with '<<<<<<< SEARCH' at offset 0 and had separator '=======', but ended prematurely"
			));
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_block_ends_prematurely_after_search_start() -> Result<()> {
		// -- Setup & Fixtures
		let original = "original content";
		let changes = MARKER_SEARCH_START.to_string(); // Only the start marker

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err());
		if let Err(e) = result {
			assert!(e.to_string().contains("missing separator marker '======='"));
		}
		Ok(())
	}

	#[test]
	fn test_support_text_apply_change_block_ends_prematurely_after_separator() -> Result<()> {
		// -- Setup & Fixtures
		let original = "original content";
		let changes = format!("{}search_text{}", MARKER_SEARCH_START, MARKER_SEARCH_REPLACE_SEP);

		// -- Exec
		let result = apply_changes(original, changes);

		// -- Check
		assert!(result.is_err());
		if let Err(e) = result {
			assert!(
				e.to_string()
					.contains("ended prematurely before replacement text or end marker")
			);
		}
		Ok(())
	}
}
// endregion: --- Tests
