use crate::support::tag::TagContentIterator;
use crate::types::Extrude;
use crate::types::TagElem;

/// Iterator that yields `TagElem` instances found within a text based on a specific tag name.
/// It uses `TagContentIterator` internally.
pub struct TagElemIter<'a> {
	input: &'a str, // Keep original input reference for extrude
	tag_names: Vec<&'a str>,
	tag_content_iter: TagContentIterator<'a>,
	extrude: Option<Extrude>,
}

impl<'a> TagElemIter<'a> {
	/// Creates a new `TagElemIter`.
	///
	/// # Arguments
	///
	/// * `input` - The string slice to search within.
	/// * `tag_name` - The name of the tag to search for (e.g., "FILE").
	/// * `extrude` - Optional configuration for extracting content outside the tags.
	///
	/// TODO: need to support tag_names
	#[cfg(test)]
	pub fn new(input: &'a str, tag_name: &'a str, extrude: Option<Extrude>) -> Self {
		Self::with_tag_names(input, &[tag_name], extrude)
	}

	pub fn with_tag_names(input: &'a str, tag_names: &[&'a str], extrude: Option<Extrude>) -> Self {
		let tag_names_vec: Vec<&'a str> = tag_names.to_vec();
		let tag_content_iter = TagContentIterator::new(input, &tag_names_vec);

		Self {
			input,
			tag_names: tag_names_vec,
			tag_content_iter,
			extrude,
		}
	}

	/// Consumes the iterator, collecting all found `TagElem`s and the concatenated
	/// content outside of the tags if `extrude` was set to `Some(Extrude::Content)`.
	///
	/// TODO: need to support tag_names
	pub fn collect_elems_and_extruded_content(self) -> (Vec<TagElem>, String) {
		let TagElemIter {
			input,
			tag_names,
			tag_content_iter: _,
			extrude,
		} = self;

		let mut elems: Vec<TagElem> = Vec::new();
		let mut extruded_content = String::new();
		let mut last_processed_idx: usize = 0;

		// We need to re-iterate using TagContentIterator to get indices for extrude
		let content_iter = TagContentIterator::new(input, &tag_names);
		let should_extrude = matches!(extrude, Some(Extrude::Content));

		for tag_content in content_iter {
			// Create the TagElem
			// Note: Skipping attrs parsing for now as per requirement.
			let elem = TagElem {
				tag: tag_content.tag_name.to_string(),
				attrs: None, // TODO: Implement attrs parsing from tag_content.attrs_raw
				content: tag_content.content.to_string(),
			};
			elems.push(elem);

			// Handle extrude if enabled
			if should_extrude {
				if tag_content.start_idx > last_processed_idx {
					extruded_content.push_str(&input[last_processed_idx..tag_content.start_idx]);
				}
				last_processed_idx = tag_content.end_idx + 1;
			}
		}

		// Append any remaining content after the last tag if extruding
		if should_extrude && last_processed_idx < input.len() {
			extruded_content.push_str(&input[last_processed_idx..]);
		}

		(
			elems,
			if should_extrude {
				extruded_content
			} else {
				String::new()
			},
		)
	}
}

impl Iterator for TagElemIter<'_> {
	type Item = TagElem;

	fn next(&mut self) -> Option<Self::Item> {
		self.tag_content_iter.next().map(|tag_content| {
			// Convert TagContent to TagElem
			// Note: Skipping attrs parsing for now.
			TagElem {
				tag: tag_content.tag_name.to_string(),
				attrs: None, // TODO: Implement attrs parsing from tag_content.attrs_raw
				content: tag_content.content.to_string(),
			}
		})
	}
}

// region:    --- Tests

#[path = "tag_elem_iter_tests.rs"]
#[cfg(test)]
mod tests;

// endregion: --- Tests
