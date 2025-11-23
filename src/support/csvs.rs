use crate::types::{CsvContent, CsvOptions};
use crate::{Error, Result};
use std::collections::HashMap;
use std::path::Path;

pub fn save_csv(path: impl AsRef<Path>, content: &CsvContent, options: Option<CsvOptions>) -> Result<()> {
	write_csv(path, content, options, false)
}

pub fn append_csv(path: impl AsRef<Path>, content: &CsvContent, options: Option<CsvOptions>) -> Result<()> {
	write_csv(path, content, options, true)
}

fn write_csv(path: impl AsRef<Path>, content: &CsvContent, options: Option<CsvOptions>, append: bool) -> Result<()> {
	let path = path.as_ref();
	let options = options.unwrap_or_default();

	// Remap headers if labels provided (clone to not modify original content)
	let mut headers = content.headers.clone();
	if let Some(labels) = &options.header_labels {
		remap_keys_to_labels(&mut headers, labels);
	}

	// Determine if we write headers
	let file_exists = path.exists();
	let write_headers = if append && file_exists {
		false
	} else {
		!options.skip_header_row.unwrap_or(false) && !headers.is_empty()
	};

	// Setup writer
	let builder = options.into_writer_builder();

	// If append, we need to open in append mode.
	let file = if append {
		simple_fs::ensure_file_dir(path)?;
		std::fs::OpenOptions::new()
			.create(true)
			.append(true)
			.open(path)
			.map_err(|e| Error::custom(format!("Failed to open CSV file for append '{}': {e}", path.display())))?
	} else {
		simple_fs::ensure_file_dir(path)?;
		std::fs::File::create(path)
			.map_err(|e| Error::custom(format!("Failed to create CSV file '{}': {e}", path.display())))?
	};

	let mut wtr = builder.from_writer(file);

	if write_headers {
		wtr.write_record(&headers)
			.map_err(|e| Error::custom(format!("Failed to write headers: {e}")))?;
	}

	for row in &content.rows {
		wtr.write_record(row)
			.map_err(|e| Error::custom(format!("Failed to write row: {e}")))?;
	}

	wtr.flush()
		.map_err(|e| Error::custom(format!("Failed to flush CSV writer: {e}")))?;

	Ok(())
}

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
			remap_labels_to_keys(&mut headers, labels);
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
		remap_labels_to_keys(&mut headers, labels);
	}

	Ok(headers)
}

/// Remap CSV headers (labels) to internal keys.
///
/// Use `header_labels` map { key: label }.
/// Function maps label -> key.
pub fn remap_labels_to_keys(headers: &mut [String], header_labels: &HashMap<String, String>) {
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

/// Remap internal keys to CSV headers (labels).
///
/// Use `header_labels` map { key: label }.
/// Function maps key -> label.
pub fn remap_keys_to_labels(headers: &mut [String], header_labels: &HashMap<String, String>) {
	// Spec: "Write: Maps internal "key" to CSV header "label""
	for header in headers.iter_mut() {
		if let Some(label) = header_labels.get(header) {
			*header = label.clone();
		}
	}
}

// region:    --- Tests

#[cfg(test)]
#[path = "csvs_tests.rs"]
mod tests;

// endregion: --- Tests
