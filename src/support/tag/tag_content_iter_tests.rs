//! Tests for the TagContentIterator.

use super::{TagContent, TagContentIterator};
// For tests, using a simple Result alias is often sufficient.
type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_support_tag_content_iter_simple() -> Result<()> {
	// -- Setup & Fixtures
	let text = "Some text <DATA>content</DATA> more text.";
	let tag_name = "DATA";

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

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
fn test_support_tag_content_iter_attrs() -> Result<()> {
	// -- Setup & Fixtures
	let text = r#"Before <FILE path="a/b.txt" id=123>File Content</FILE> After"#;
	let tag_name = "FILE";

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

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
fn test_support_tag_content_iter_attrs_with_newline() -> Result<()> {
	// -- Setup & Fixtures
	let text = "Before <FILE \npath=\"a/b.txt\"\n id=123>File Content</FILE> After";
	let tag_name = "FILE";

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

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
fn test_support_tag_content_iter_multiple() -> Result<()> {
	// -- Setup & Fixtures
	let text = "Data: <ITEM>one</ITEM>, <ITEM key=val>two</ITEM>.";
	let tag_name = "ITEM";

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

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
fn test_support_tag_content_iter_no_tags() -> Result<()> {
	// -- Setup & Fixtures
	let text = "Just plain text without any tags.";
	let tag_name = "MARKER";

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

	// -- Check
	assert!(tags.is_empty());

	Ok(())
}

#[test]
fn test_support_tag_content_iter_empty_content() -> Result<()> {
	// -- Setup & Fixtures
	let text = "<EMPTY></EMPTY>";
	let tag_name = "EMPTY";

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

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
fn test_support_tag_content_iter_nested_like() -> Result<()> {
	// -- Setup & Fixtures
	let text = "<OUTER>outer <INNER>inner</INNER> outer</OUTER>";
	let tag_name_outer = "OUTER";
	let tag_name_inner = "INNER";

	// -- Exec Outer
	let tags_outer: Vec<TagContent> = TagContentIterator::new(text, &[tag_name_outer]).collect();
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
	let tags_inner: Vec<TagContent> = TagContentIterator::new(text, &[tag_name_inner]).collect();
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
fn test_support_tag_content_iter_malformed_open() -> Result<()> {
	// -- Setup & Fixtures
	let text = "<MARKER oops </MARKER>"; // Missing '>'
	let tag_name = "MARKER";

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

	// -- Check
	// The current implementation stops if '>' isn't found for the opening tag.
	assert!(tags.is_empty());

	Ok(())
}

#[test]
fn test_support_tag_content_iter_unclosed() -> Result<()> {
	// -- Setup & Fixtures
	let text = "<MARKER>content"; // Missing closing tag
	let tag_name = "MARKER";

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

	// -- Check
	// The current implementation stops if the closing tag isn't found.
	assert!(tags.is_empty());

	Ok(())
}

#[test]
fn test_support_tag_content_iter_edges() -> Result<()> {
	// -- Setup & Fixtures
	let text = "<START>at start</START>middle<END>at end</END>";
	let tag_name_start = "START";
	let tag_name_end = "END";

	// -- Exec Start
	let tags_start: Vec<TagContent> = TagContentIterator::new(text, &[tag_name_start]).collect();
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
	let tags_end: Vec<TagContent> = TagContentIterator::new(text, &[tag_name_end]).collect();
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
fn test_support_tag_content_iter_incorrect_tag_name() -> Result<()> {
	// -- Setup & Fixtures
	let text = "<MARKERX>content</MARKERX>";
	let tag_name = "MARKER"; // Searching for MARKER, not MARKERX

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

	// -- Check
	assert!(tags.is_empty());

	Ok(())
}

#[test]
fn test_support_tag_content_iter_tag_name_prefix_check() -> Result<()> {
	// -- Setup & Fixtures
	let text = "<TAG_EXTRA>extra</TAG_EXTRA><TAG>real</TAG>";
	let tag_name = "TAG";

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &[tag_name]).collect();

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

#[test]
fn test_support_tag_content_iter_multiple_tag_names() -> Result<()> {
	// -- Setup & Fixtures
	let text = "Alpha <ONE>first</ONE> Beta <TWO attr=ok>second</TWO> Gamma";
	let tag_names = ["ONE", "TWO"];

	// -- Exec
	let tags: Vec<TagContent> = TagContentIterator::new(text, &tag_names).collect();

	// -- Check
	assert_eq!(tags.len(), 2);
	assert_eq!(
		tags[0],
		TagContent {
			tag_name: "ONE",
			attrs_raw: None,
			content: "first",
			start_idx: 6,
			end_idx: 21,
		}
	);
	assert_eq!(
		tags[1],
		TagContent {
			tag_name: "TWO",
			attrs_raw: Some("attr=ok"),
			content: "second",
			start_idx: 28,
			end_idx: 52,
		}
	);

	Ok(())
}
