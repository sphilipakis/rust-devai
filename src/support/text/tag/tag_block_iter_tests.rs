//! Tests for the TagBlockIter and its extrude functionality.

use crate::support::text::TagBlockIter;
use crate::types::Extrude;
use crate::types::TagElem; // Make sure TagBlock derives Default, PartialEq, Debug

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_support_text_tag_block_iter_simple() -> Result<()> {
	// -- Setup & Fixtures
	let text = "Some text <DATA>content1</DATA> more text <DATA>content2</DATA> final.";
	let tag_name = "DATA";

	// -- Exec
	let iter = TagBlockIter::new(text, tag_name, None);
	let blocks: Vec<TagElem> = iter.collect();

	// -- Check
	assert_eq!(blocks.len(), 2);
	assert_eq!(
		blocks[0],
		TagElem {
			tag: "DATA".to_string(),
			attrs: None,
			content: "content1".to_string()
		}
	);
	assert_eq!(
		blocks[1],
		TagElem {
			tag: "DATA".to_string(),
			attrs: None,
			content: "content2".to_string()
		}
	);

	Ok(())
}

#[test]
fn test_support_text_tag_block_iter_no_tags() -> Result<()> {
	// -- Setup & Fixtures
	let text = "Some text without tags.";
	let tag_name = "DATA";

	// -- Exec
	let iter = TagBlockIter::new(text, tag_name, None);
	let blocks: Vec<TagElem> = iter.collect();

	// -- Check
	assert!(blocks.is_empty());

	Ok(())
}

#[test]
fn test_support_text_tag_block_iter_collect_extrude_simple() -> Result<()> {
	// -- Setup & Fixtures
	let text = "Prefix <DATA>content1</DATA> Infix <DATA>content2</DATA> Suffix";
	let tag_name = "DATA";

	// -- Exec
	let iter = TagBlockIter::new(text, tag_name, Some(Extrude::Content));
	let (blocks, extruded) = iter.collect_blocks_and_extruded_content();

	// -- Check Blocks
	assert_eq!(blocks.len(), 2);
	assert_eq!(
		blocks[0],
		TagElem {
			tag: "DATA".to_string(),
			attrs: None,
			content: "content1".to_string()
		}
	);
	assert_eq!(
		blocks[1],
		TagElem {
			tag: "DATA".to_string(),
			attrs: None,
			content: "content2".to_string()
		}
	);

	// -- Check Extruded Content
	assert_eq!(extruded, "Prefix  Infix  Suffix");

	Ok(())
}

#[test]
fn test_support_text_tag_block_iter_collect_extrude_no_tags() -> Result<()> {
	// -- Setup & Fixtures
	let text = "Just plain text.";
	let tag_name = "DATA";

	// -- Exec
	let iter = TagBlockIter::new(text, tag_name, Some(Extrude::Content));
	let (blocks, extruded) = iter.collect_blocks_and_extruded_content();

	// -- Check Blocks
	assert!(blocks.is_empty());

	// -- Check Extruded Content
	assert_eq!(extruded, "Just plain text.");

	Ok(())
}

#[test]
fn test_support_text_tag_block_iter_collect_extrude_edges() -> Result<()> {
	// -- Setup & Fixtures
	let text = "<START>at start</START>middle<END>at end</END>";
	let tag_name_start = "START";
	let tag_name_end = "END";

	// -- Exec Start Tag
	let iter_start = TagBlockIter::new(text, tag_name_start, Some(Extrude::Content));
	let (blocks_start, extruded_start) = iter_start.collect_blocks_and_extruded_content();

	// -- Check Start Tag
	assert_eq!(blocks_start.len(), 1);
	assert_eq!(blocks_start[0].content, "at start");
	assert_eq!(extruded_start, "middle<END>at end</END>"); // Extrudes everything not matching START

	// -- Exec End Tag
	let iter_end = TagBlockIter::new(text, tag_name_end, Some(Extrude::Content));
	let (blocks_end, extruded_end) = iter_end.collect_blocks_and_extruded_content();

	// -- Check End Tag
	assert_eq!(blocks_end.len(), 1);
	assert_eq!(blocks_end[0].content, "at end");
	assert_eq!(extruded_end, "<START>at start</START>middle"); // Extrudes everything not matching END

	Ok(())
}

#[test]
fn test_support_text_tag_block_iter_collect_extrude_empty_input() -> Result<()> {
	// -- Setup & Fixtures
	let text = "";
	let tag_name = "DATA";

	// -- Exec
	let iter = TagBlockIter::new(text, tag_name, Some(Extrude::Content));
	let (blocks, extruded) = iter.collect_blocks_and_extruded_content();

	// -- Check
	assert!(blocks.is_empty());
	assert!(extruded.is_empty());

	Ok(())
}

#[test]
fn test_support_text_tag_block_iter_collect_extrude_only_tags() -> Result<()> {
	// -- Setup & Fixtures
	let text = "<D1>c1</D1><D2>c2</D2>";
	let tag_name = "D1";

	// -- Exec
	let iter = TagBlockIter::new(text, tag_name, Some(Extrude::Content));
	let (blocks, extruded) = iter.collect_blocks_and_extruded_content();

	// -- Check
	assert_eq!(blocks.len(), 1);
	assert_eq!(blocks[0].content, "c1");
	assert_eq!(extruded, "<D2>c2</D2>"); // Extrudes the D2 tag part

	Ok(())
}
