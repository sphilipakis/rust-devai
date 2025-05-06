use crate::support::common::Extrude;
use crate::support::text::tag::TagContentIterator;
use crate::types::TagBlock;

/// Iterator that yields `TagBlock` instances found within a text based on a specific tag name.
/// It uses `TagContentIterator` internally.
pub struct TagBlockIter<'a> {
	input: &'a str, // Keep original input reference for extrude
	tag_name: &'a str,
	tag_content_iter: TagContentIterator<'a>,
	extrude: Option<Extrude>,
}

impl<'a> TagBlockIter<'a> {
	/// Creates a new `TagBlockIter`.
	///
	/// # Arguments
	///
	/// * `input` - The string slice to search within.
	/// * `tag_name` - The name of the tag to search for (e.g., "FILE").
	/// * `extrude` - Optional configuration for extracting content outside the tags.
	pub fn new(input: &'a str, tag_name: &'a str, extrude: Option<Extrude>) -> Self {
		Self {
			input,
			tag_name,
			tag_content_iter: TagContentIterator::new(input, tag_name),
			extrude,
		}
	}

	/// Consumes the iterator, collecting all found `TagBlock`s and the concatenated
	/// content outside of the tags if `extrude` was set to `Some(Extrude::Content)`.
	pub fn collect_blocks_and_extruded_content(self) -> (Vec<TagBlock>, String) {
		let mut blocks: Vec<TagBlock> = Vec::new();
		let mut extruded_content = String::new();
		let mut last_processed_idx: usize = 0;

		// We need to re-iterate using TagContentIterator to get indices for extrude
		let content_iter = TagContentIterator::new(self.input, self.tag_name);

		for tag_content in content_iter {
			// Create the TagBlock
			// Note: Skipping attrs parsing for now as per requirement.
			let block = TagBlock {
				name: tag_content.tag_name.to_string(),
				attrs: None, // TODO: Implement attrs parsing from tag_content.attrs_raw
				content: tag_content.content.to_string(),
			};
			blocks.push(block);

			// Handle extrude if enabled
			if let Some(Extrude::Content) = self.extrude {
				if tag_content.start_idx > last_processed_idx {
					extruded_content.push_str(&self.input[last_processed_idx..tag_content.start_idx]);
				}
				last_processed_idx = tag_content.end_idx + 1;
			}
		}

		// Append any remaining content after the last tag if extruding
		if let Some(Extrude::Content) = self.extrude {
			if last_processed_idx < self.input.len() {
				extruded_content.push_str(&self.input[last_processed_idx..]);
			}
		}

		(blocks, extruded_content)
	}
}

impl Iterator for TagBlockIter<'_> {
	type Item = TagBlock;

	fn next(&mut self) -> Option<Self::Item> {
		self.tag_content_iter.next().map(|tag_content| {
			// Convert TagContent to TagBlock
			// Note: Skipping attrs parsing for now.
			TagBlock {
				name: tag_content.tag_name.to_string(),
				attrs: None, // TODO: Implement attrs parsing from tag_content.attrs_raw
				content: tag_content.content.to_string(),
			}
		})
	}
}

// region:    --- Tests

#[path = "../../../_tests/tests_support_text_tag_block_iter.rs"]
#[cfg(test)]
mod tests;

// endregion: --- Tests
