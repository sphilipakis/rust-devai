use crate::types::{CsvContent, CsvOptions};
use crate::{Error, Result};
use std::collections::HashMap;
use std::path::Path;

pub fn values_to_csv_row(values: &[String], options: Option<CsvOptions>) -> Result<String> {
	let options = options.unwrap_or_default();
	let mut builder = options.into_writer_builder();
	// We force has_headers(false) because we are writing a single row of values,
	// and we do not want the writer to treat the first call as headers (if we were to use higher level APIs),
	// although write_record treats input as record regardless.
	builder.has_headers(false);

	let mut wtr = builder.from_writer(Vec::new());

	wtr.write_record(values)
		.map_err(|e| Error::custom(format!("Failed to write CSV record: {e}")))?;

	let bytes = wtr
		.into_inner()
		.map_err(|e| Error::custom(format!("Failed to retrieve CSV buffer: {e}")))?;

	let s =
		String::from_utf8(bytes).map_err(|e| Error::custom(format!("Failed to convert CSV buffer to UTF-8: {e}")))?;

	// Trim the trailing newline which is added by the CSV writer by default.
	// We only remove one occurrence of the terminator (\n or \r\n).
	let s = if let Some(stripped) = s.strip_suffix('\n') {
		if let Some(stripped_cr) = stripped.strip_suffix('\r') {
			stripped_cr.to_string()
		} else {
			stripped.to_string()
		}
	} else {
		s
	};

	Ok(s)
}

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
	let mut options = options.unwrap_or_default();
	let has_header = options.has_header.unwrap_or(true);
	let skip_empty_lines = options.skip_empty_lines.unwrap_or(true);

	let header_labels = options.header_labels.take();

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

		if let Some(labels) = &header_labels {
			remap_headers(&mut headers, labels);
		}
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

	parse_csv(&content, options)
		.map_err(|e| Error::custom(format!("Failed to parse CSV file '{}': {}", path.display(), e)))
}

pub fn load_csv_headers(path: impl AsRef<Path>, options: Option<CsvOptions>) -> Result<Vec<String>> {
	let path = path.as_ref();
	let mut options = options.unwrap_or_default();

	let header_labels = options.header_labels.take();

	let mut builder = options.into_reader_builder();
	builder.has_headers(true);

	let file = std::fs::File::open(path)
		.map_err(|e| Error::custom(format!("Failed to open CSV file '{}': {e}", path.display())))?;
	let mut rdr = builder.from_reader(file);

	let headers = rdr.headers().map_err(|e| {
		Error::custom(format!(
			"Failed to read CSV headers from file '{}': {e}",
			path.display()
		))
	})?;

	let mut headers: Vec<String> = headers.iter().map(|s| s.to_string()).collect();

	if let Some(labels) = &header_labels {
		remap_headers(&mut headers, labels);
	}

	Ok(headers)
}

fn remap_headers(headers: &mut [String], header_labels: &HashMap<String, String>) {
	// Build reverse mapping: label -> key
	// If multiple keys map to the same label, the last one wins (or arbitrary).
	// Spec: "If a CSV header matches a `label` (value) in the map, it is renamed to the corresponding `key`."
	let label_to_key: HashMap<&String, &String> = header_labels.iter().map(|(k, v)| (v, k)).collect();

	for header in headers.iter_mut() {
		if let Some(key) = label_to_key.get(header) {
			*header = key.to_string();
		}
	}
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
	fn test_support_files_csv_to_csv_row_simple() -> Result<()> {
		let row = values_to_csv_row(&["a".to_string(), "b,c".to_string(), "d".to_string()], None)?;
		assert_eq!(row, r#"a,"b,c",d"#);
		Ok(())
	}

	#[test]
	fn test_support_files_csv_to_csv_row_custom_delimiter() -> Result<()> {
		let options = CsvOptions {
			delimiter: Some(";".to_string()),
			..Default::default()
		};
		let row = values_to_csv_row(&["a".to_string(), "b;c".to_string(), "d".to_string()], Some(options))?;
		assert_eq!(row, r#"a;"b;c";d"#);
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

		let expected_content = vec![vec!["1", "Alice", "alice@example.com"], vec!["2", "Bob", "bob@example.com"]];
		let content_as_strs: Vec<Vec<&str>> = res.rows.iter().map(|row| row.x_as_strs()).collect();
		assert_eq!(content_as_strs, expected_content);

		Ok(())
	}

	#[test]
	fn test_support_files_csv_load_no_header() -> Result<()> {
		// -- Setup & Fixtures
		let path = SPath::new("tests-data/sandbox-01/example.csv");

		// -- Exec
		let res = load_csv(
			&path,
			Some(CsvOptions {
				has_header: Some(false),
				..Default::default()
			}),
		)?;

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

	#[test]
	fn test_support_files_csv_load_with_header_labels() -> Result<()> {
		// -- Setup & Fixtures
		let content = "ID Code,Full Name,Contact Email\n1,Alice,alice@x.com";
		let mut labels = HashMap::new();
		labels.insert("id".to_string(), "ID Code".to_string());
		labels.insert("name".to_string(), "Full Name".to_string());
		// "Contact Email" not mapped, should remain as is.

		let options = CsvOptions {
			header_labels: Some(labels),
			..Default::default()
		};

		// -- Exec
		let res = parse_csv(content, Some(options))?;

		// -- Check
		let expected_headers = vec!["id", "name", "Contact Email"];
		assert_eq!(res.headers, expected_headers);
		assert_eq!(res.rows.len(), 1);
		assert_eq!(res.rows[0], vec!["1", "Alice", "alice@x.com"]);

		Ok(())
	}

	#[test]
	fn test_support_files_csv_load_headers_remapped() -> Result<()> {
		// -- Setup & Fixtures
		// Content: id,name,email
		let path = SPath::new("tests-data/sandbox-01/example.csv");

		let mut labels = HashMap::new();
		labels.insert("ID".to_string(), "id".to_string()); // map "id" -> "ID"
		// "name" and "email" unmapped

		let options = CsvOptions {
			header_labels: Some(labels),
			..Default::default()
		};

		// -- Exec
		let headers = load_csv_headers(&path, Some(options))?;

		// -- Check
		// Original: ["id", "name", "email"]
		// Remapped: ["ID", "name", "email"]
		let expected = vec!["ID", "name", "email"];
		assert_eq!(headers, expected);

		Ok(())
	}
}

// endregion: --- Tests
