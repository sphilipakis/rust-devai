use super::*;
use crate::Error;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

// #[test]
// fn test_packer_support_normalize_version_simple() -> Result<()> {
// 	assert_eq!(normalize_version("1.0.0"), "1-0-0");
// 	assert_eq!(normalize_version("1.0-alpha"), "1-0-alpha");
// 	assert_eq!(normalize_version("1.0 beta"), "1-0-beta");
// 	assert_eq!(normalize_version("1.0-beta-2"), "1-0-beta-2");
// 	assert_eq!(normalize_version("1.0--beta--2"), "1-0-beta-2");
// 	assert_eq!(normalize_version("v1.0.0_rc1"), "v1-0-0-rc1");
// 	assert_eq!(normalize_version("1.0.0!@#$%^&*()"), "1-0-0");

// 	Ok(())
// }

#[test]
fn test_packer_support_validate_version_update_simple() -> Result<()> {
	use std::cmp::Ordering;

	// Test case: New version is greater than installed
	assert_eq!(validate_version_update("1.0.0", "1.0.1")?, Ordering::Greater);
	assert_eq!(validate_version_update("1.0.0", "1.1.0")?, Ordering::Greater);
	assert_eq!(validate_version_update("1.0.0", "2.0.0")?, Ordering::Greater);

	// Test case: New version is equal to installed
	assert_eq!(validate_version_update("1.0.0", "1.0.0")?, Ordering::Equal);

	// Test case: New version is less than installed
	assert_eq!(validate_version_update("1.0.1", "1.0.0")?, Ordering::Less);

	// Test with leading 'v'
	assert_eq!(validate_version_update("v1.0.0", "1.0.1")?, Ordering::Greater);
	assert_eq!(validate_version_update("1.0.0", "v1.0.1")?, Ordering::Greater);

	// Test with invalid versions (string comparison fallback)
	assert_eq!(validate_version_update("a", "b")?, Ordering::Greater);
	assert_eq!(validate_version_update("b", "a")?, Ordering::Less);
	assert_eq!(validate_version_update("a", "a")?, Ordering::Equal);

	Ok(())
}

#[test]
fn test_packer_support_validate_version_for_install_valid() -> Result<()> {
	// Test valid versions
	assert!(validate_version_for_install("0.1.0").is_ok());
	assert!(validate_version_for_install("1.0.0").is_ok());
	assert!(validate_version_for_install("0.1.1-alpha.1").is_ok());
	assert!(validate_version_for_install("0.1.1-beta.123").is_ok());
	assert!(validate_version_for_install("0.1.1-rc.1.2").is_ok());
	assert!(validate_version_for_install("v1.0.0-alpha.1").is_ok());

	Ok(())
}

#[test]
fn test_packer_support_validate_version_for_install_invalid() -> Result<()> {
	// Test invalid versions
	let err = validate_version_for_install("0.1.1-alpha").unwrap_err();
	match err {
		Error::InvalidPrereleaseFormat { version } => {
			assert_eq!(version, "0.1.1-alpha");
		}
		_ => panic!("Expected InvalidPrereleaseFormat error"),
	}

	let err = validate_version_for_install("0.1.1-alpha.text").unwrap_err();
	match err {
		Error::InvalidPrereleaseFormat { version } => {
			assert_eq!(version, "0.1.1-alpha.text");
		}
		_ => panic!("Expected InvalidPrereleaseFormat error"),
	}

	let err = validate_version_for_install("0.1.1-alpha.1.some").unwrap_err();
	match err {
		Error::InvalidPrereleaseFormat { version } => {
			assert_eq!(version, "0.1.1-alpha.1.some");
		}
		_ => panic!("Expected InvalidPrereleaseFormat error"),
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_repo() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "pro@coder";

	// -- Exec
	let pack_uri = PackUri::parse(uri);

	// -- Check
	assert!(matches!(pack_uri, PackUri::RepoPack(_)));
	if let PackUri::RepoPack(identity) = &pack_uri {
		assert_eq!(identity.namespace, "pro");
		assert_eq!(identity.name, "coder");
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_http() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "https://example.com/some-pack.aipack";

	// -- Exec
	let pack_uri = PackUri::parse(uri);

	// -- Check
	assert!(matches!(pack_uri, PackUri::HttpLink(_)));
	if let PackUri::HttpLink(url) = &pack_uri {
		assert_eq!(url, "https://example.com/some-pack.aipack");
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_local() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "./path/to/pack.aipack";

	// -- Exec
	let pack_uri = PackUri::parse(uri);

	// -- Check
	assert!(matches!(pack_uri, PackUri::LocalPath(_)));
	if let PackUri::LocalPath(path) = &pack_uri {
		assert_eq!(path, "./path/to/pack.aipack");
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_display() -> Result<()> {
	// -- Setup & Fixtures
	let data = [
		("pro@coder", "pro@coder"),
		(
			"https://example.com/pack.aipack",
			"URL 'https://example.com/pack.aipack'",
		),
		("./local.aipack", "local file './local.aipack'"),
	];

	// -- Exec & Check
	for (input, expected_display) in data {
		let pack_uri = PackUri::parse(input);
		assert_eq!(pack_uri.to_string(), expected_display, "Input: {input}");
	}

	Ok(())
}
