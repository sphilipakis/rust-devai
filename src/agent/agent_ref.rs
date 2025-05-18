//! This module defines the AgentRef enum used to reference an agent either as a local file path
//! or as a PackRef, which can be parsed from a string using the '@' delimiter.

use crate::Result;
use crate::types::{LocalPackRef, PackRef};
use std::str::FromStr;

/// AgentRef represents a reference to an agent.
/// It can either be a LocalPath (a direct file path) or a PackRef (a parsed representation).
#[derive(Debug, Clone)]
pub enum PartialAgentRef {
	LocalPath(String),
	PackRef(PackRef),
}

/// Constructors
impl PartialAgentRef {
	/// Create a new AgentRef from an input string.
	///
	/// The function determines the type of AgentRef based on the presence of the '@' character.
	///
	/// If the input string contains '@':
	///   - It is parsed into a PackRef.
	///   - Example: "pro@coder" will be parsed as
	/// ```
	/// PackRef {
	///     namespace: Some("jc"),
	///     pack_name: "coder",
	///     sub_path: None
	/// }
	///````
	///   - If a subpath is provided (e.g., "pro@coder/explain"), the sub_path field is set.
	///
	/// If the input string does not contain '@':
	///   - It is treated as a local file path.
	///
	pub fn new(input: &str) -> Result<Self> {
		// Check if the input contains the delimiter '@'
		if input.contains('@') {
			let partial_pack_ref = PackRef::from_str(input)?;
			// Return a PackRef wrapped in the AgentRef enum
			Ok(PartialAgentRef::PackRef(partial_pack_ref))
		} else {
			// If no '@' is found, treat the input as a local file path and return it as LocalPath.
			Ok(PartialAgentRef::LocalPath(input.to_string()))
		}
	}
}

/// Implement the Display trait for AgentRef
impl std::fmt::Display for PartialAgentRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PartialAgentRef::LocalPath(path) => write!(f, "{}", path),
			PartialAgentRef::PackRef(pack_ref) => {
				write!(f, "{}", pack_ref)?;
				Ok(())
			}
		}
	}
}

// region:    --- Final AgentRef

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum AgentRef {
	LocalPath(String),
	PackRef(LocalPackRef),
}

// endregion: --- Final AgentRef

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;
	use crate::_test_support::assert_contains;

	#[test]
	fn test_agent_ref_new_localpath() -> Result<()> {
		// -- Setup & Fixtures
		let input = "path/to/local/file.rs";

		// -- Exec
		let agent_ref = PartialAgentRef::new(input)?;

		// -- Check
		match agent_ref {
			PartialAgentRef::LocalPath(ref path) => {
				assert_eq!(path, input, "The local path should match the input string.");
			}
			_ => panic!("Expected AgentRef::LocalPath but got a different variant."),
		}

		Ok(())
	}

	#[test]
	fn test_agent_ref_new_packref_without_subpath() -> Result<()> {
		// -- Setup & Fixtures
		let input = "pro@coder";

		// -- Exec
		let agent_ref = PartialAgentRef::new(input)?;

		// -- Check
		match agent_ref {
			PartialAgentRef::PackRef(ref pack_ref) => {
				assert_eq!(pack_ref.namespace, "pro", "Namespace should be 'pro'.");
				assert_eq!(pack_ref.name, "coder", "Pack name should be 'coder'.");
				assert!(pack_ref.sub_path.is_none(), "Sub path should be None.");
			}
			_ => panic!("Expected AgentRef::PackRef but got a different variant."),
		}

		Ok(())
	}

	#[test]
	fn test_agent_ref_invalid() -> Result<()> {
		// -- Setup & Fixtures
		let input = " jc @ coder/example/path ";
		// note: the input contains spaces which should be trimmed

		// -- Exec
		let err = PartialAgentRef::new(input).err().ok_or("Should return error")?;

		// -- Check
		assert_contains(
			&err.to_string(),
			"Pack namespace can only contain alphanumeric characters,",
		);

		Ok(())
	}
}

// endregion: --- Tests
