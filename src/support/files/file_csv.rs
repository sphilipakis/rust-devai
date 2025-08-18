use crate::{Error, Result};
use csv::ReaderBuilder;
use simple_fs::SPath;

pub fn load_csv_headers(path: &SPath) -> Result<Vec<String>> {
	let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path).map_err(Error::custom)?;

	let headers = rdr.headers().map_err(Error::custom)?;
	Ok(headers.iter().map(|s| s.to_string()).collect())
}

pub struct LoadCsvResponse {
	pub headers: Vec<String>,
	pub content: Vec<Vec<String>>,
}

pub fn load_csv(path: &SPath, with_headers: Option<bool>) -> Result<LoadCsvResponse> {
	let with_headers = with_headers.unwrap_or(true);
	let mut rdr = ReaderBuilder::new()
		.has_headers(with_headers)
		.from_path(path)
		.map_err(Error::custom)?;

	let headers = if with_headers {
		let hdrs = rdr.headers().map_err(Error::custom)?;
		hdrs.iter().map(|s| s.to_string()).collect()
	} else {
		Vec::new()
	};

	let mut content: Vec<Vec<String>> = Vec::new();
	for rec in rdr.records() {
		let rec = rec.map_err(Error::custom)?;
		content.push(rec.iter().map(|s| s.to_string()).collect());
	}

	Ok(LoadCsvResponse { headers, content })
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
		let headers = load_csv_headers(&path)?;

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

		let expected_content = vec![vec!["1", "Alice", "alice@example.com"], vec!["2", "Bob", "bob@example.com"]];
		let content_as_strs: Vec<Vec<&str>> = res.content.iter().map(|row| row.x_as_strs()).collect();
		assert_eq!(content_as_strs, expected_content);

		Ok(())
	}

	#[test]
	fn test_support_files_csv_load_no_header() -> Result<()> {
		// -- Setup & Fixtures
		let path = SPath::new("tests-data/sandbox-01/example.csv");

		// -- Exec
		let res = load_csv(&path, Some(false))?;

		// -- Check
		assert!(res.headers.is_empty(), "Headers should be empty when no_header is true");

		let expected_content = vec![
			vec!["id", "name", "email"],
			vec!["1", "Alice", "alice@example.com"],
			vec!["2", "Bob", "bob@example.com"],
		];
		assert_eq!(res.content, expected_content);

		Ok(())
	}
}

// endregion: --- Tests
