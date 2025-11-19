use crate::types::{CsvContent, CsvOptions};
use crate::{Error, Result};
use std::path::Path;

pub fn parse_csv_row(row: &str, options: Option<CsvOptions>) -> Result<Vec<String>> {
	let options = options.unwrap_or_default();
	let mut builder = options.into_reader_builder();
	builder.has_headers(false).comment(None).flexible(true);

	let mut rdr = builder.from_reader(row.as_bytes());

	let mut iter = rdr.records();
	if let Some(result) = iter.next() {
		let record = result.map_err(|e| Error::custom(format!("Failed to parse CSV row: {e}")))?;
		Ok(record.iter().map(|s| s.to_string()).collect())
	} else {
		Ok(Vec::new())
	}
}

pub fn parse_csv(content: &str, options: Option<CsvOptions>) -> Result<CsvContent> {
	let options = options.unwrap_or_default();
	let has_header = options.has_header.unwrap_or(true);
	let skip_empty_lines = options.skip_empty_lines.unwrap_or(true);

	let mut builder = options.into_reader_builder();
	// We set has_headers explicitly to handle the extraction logic below correctly
	builder.has_headers(has_header).flexible(true);

	let mut rdr = builder.from_reader(content.as_bytes());

	let mut headers = Vec::new();
	if has_header {
		let hdr = rdr
			.headers()
			.map_err(|e| Error::custom(format!("Failed to read CSV headers: {e}")))?;
		headers = hdr.iter().map(|s| s.to_string()).collect();
	}

	let mut rows = Vec::new();
	for result in rdr.records() {
		let record = result.map_err(|e| Error::custom(format!("Failed to read CSV record: {e}")))?;
		if skip_empty_lines && record.iter().all(|s| s.trim().is_empty()) {
			continue;
		}
		rows.push(record.iter().map(|s| s.to_string()).collect());
	}

	Ok(CsvContent { headers, rows })
}

pub fn load_csv(path: impl AsRef<Path>, options: Option<CsvOptions>) -> Result<CsvContent> {
	let path = path.as_ref();
	let content = simple_fs::read_to_string(path)?;

	parse_csv(&content, options).map_err(|e| {
		Error::custom(format!(
			"Failed to parse CSV file '{}': {}",
			path.display(),
			e
		))
	})
}

pub fn load_csv_headers(path: impl AsRef<Path>, options: Option<CsvOptions>) -> Result<Vec<String>> {
	let path = path.as_ref();
	let options = options.unwrap_or_default();

	let mut builder = options.into_reader_builder();
	builder.has_headers(true);

	let file = std::fs::File::open(path)
		.map_err(|e| Error::custom(format!("Failed to open CSV file '{}': {e}", path.display())))?;
	let mut rdr = builder.from_reader(file);

	let headers = rdr
		.headers()
		.map_err(|e| Error::custom(format!("Failed to read CSV headers from file '{}': {e}", path.display())))?;

	Ok(headers.iter().map(|s| s.to_string()).collect())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;
	use crate::support::AsStrsExt as _;
	use simple_fs::SPath;

	#[test]
	fn test_support_files_csv_load_headers_simple() -> Result<()> {
		// -- Setup & Fixtures
		let path = SPath::new("tests-data/sandbox-01/example.csv");

		// -- Exec
		let headers = load_csv_headers(&path, None)?;

		// -- Check
		let expected = vec!["id", "name", "email"];
		assert_eq!(headers.x_as_strs(), expected);

		Ok(())
	}

	#[test]
	fn test_support_files_csv_load_simple() -> Result<()> {
		// -- Setup & Fixtures
		let path = SPath::new("tests-data/sandbox-01/example.csv");

		// -- Exec
		let res = load_csv(&path, None)?;

		// -- Check
		let expected_headers = vec!["id", "name", "email"];
		assert_eq!(res.headers.x_as_strs(), expected_headers);

		let expected_content = vec![
			vec!["1", "Alice", "alice@example.com"],
			vec!["2", "Bob", "bob@example.com"],
		];
		let content_as_strs: Vec<Vec<&str>> = res.rows.iter().map(|row| row.x_as_strs()).collect();
		assert_eq!(content_as_strs, expected_content);

		Ok(())
	}

	#[test]
	fn test_support_files_csv_load_no_header() -> Result<()> {
		// -- Setup & Fixtures
		let path = SPath::new("tests-data/sandbox-01/example.csv");

		// -- Exec
		let res = load_csv(&path, Some(CsvOptions {
			has_header: Some(false),
			..Default::default()
		}))?;

		// -- Check
		assert!(res.headers.is_empty(), "Headers should be empty when no_header is true");

		let expected_content = vec![
			vec!["id", "name", "email"],
			vec!["1", "Alice", "alice@example.com"],
			vec!["2", "Bob", "bob@example.com"],
		];

		let content_as_strs: Vec<Vec<&str>> = res.rows.iter().map(|row| row.x_as_strs()).collect();
		assert_eq!(content_as_strs, expected_content);

		Ok(())
	}
}

// endregion: --- Tests
