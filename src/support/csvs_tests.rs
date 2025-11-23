type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

use super::*;
use crate::support::AsStrsExt as _;
use simple_fs::SPath;

#[test]
fn test_support_csv_load_headers_simple() -> Result<()> {
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
fn test_support_csv_to_csv_row_simple() -> Result<()> {
	let row = values_to_csv_row(&["a".to_string(), "b,c".to_string(), "d".to_string()], None)?;
	assert_eq!(row, r#"a,"b,c",d"#);
	Ok(())
}

#[test]
fn test_support_csv_to_csv_row_custom_delimiter() -> Result<()> {
	let options = CsvOptions {
		delimiter: Some(";".to_string()),
		..Default::default()
	};
	let row = values_to_csv_row(&["a".to_string(), "b;c".to_string(), "d".to_string()], Some(options))?;
	assert_eq!(row, r#"a;"b;c";d"#);
	Ok(())
}

#[test]
fn test_support_csv_load_simple() -> Result<()> {
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
fn test_support_csv_load_no_header() -> Result<()> {
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
fn test_support_csv_load_with_header_labels() -> Result<()> {
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
fn test_support_csv_load_headers_remapped() -> Result<()> {
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
