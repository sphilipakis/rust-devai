// region:    --- PartialPackRef

use crate::dir_context::{PackDir, RepoKind};
use crate::pack::PackIdentity;
use crate::{Error, Result};
use simple_fs::SPath;
use std::str::FromStr;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PackRefSubPathScope {
	#[default]
	BaseInstalled,
	BaseSupport,
	WorkspaceSupport,
}

/// PartialPackRef represents a resource reference to a pack resource.
/// It has not be resolved yet
/// For example, a string like "pro@coder/explain" will be parsed into:
///     - namespace: Some("pro")
///     - name: "coder"
///     - sub_path_scope: BaseInstalled (by default)
///     - sub_path: Some("explain")
///
/// We also can also get the "support" dir for a pack. This allows to store/read support file during execution.
/// The scope of the `support` is given with the `$base` or `$workspace`
/// - `pro@coder$base/some-file.txt` - `~/.aipack-base/support/pack/pro/coder/some-file.txt`
/// - `pro@coder$workspace/some-file.txt` - `.workspace/support/pack/pro/coder/some-file.txt`
#[derive(Debug, Clone, PartialEq)]
pub struct PackRef {
	pub namespace: String,
	pub name: String,
	pub sub_path_scope: PackRefSubPathScope,
	pub sub_path: Option<String>,
}

/// Implement the FromStr trait for PartialPackRef to parse string references
impl FromStr for PackRef {
	type Err = Error;

	fn from_str(full_ref: &str) -> Result<Self> {
		let parts: Vec<&str> = full_ref.split('@').collect();

		let (namespace_str, name_and_path_str) = match parts.len() {
			1 => {
				// Format: name... (no '@')
				// Catches the case of input ""
				return Err(Error::InvalidPackIdentity {
					origin_path: full_ref.to_string(),
					cause: "No '@' sign".to_string(),
				});
			}
			2 => {
				// Format: namespace@name...
				let ns = parts[0];
				let rest = parts[1];
				if ns.is_empty() {
					// Catches cases like "@name"
					return Err(Error::custom(format!(
						"Invalid pack reference format: '{}'. Namespace cannot be empty when '@' is present.",
						full_ref
					)));
				}
				if rest.is_empty() {
					// Catches cases like "ns@"
					return Err(Error::custom(format!(
						"Invalid pack reference format: '{}'. Pack name/path part cannot be empty after '@'.",
						full_ref
					)));
				}
				// Validate namespace characters early
				PackIdentity::validate_namespace(ns)?;
				(ns, rest)
			}
			_ => {
				// More than one '@' (e.g., "ns@name@extra", "ns@@name")
				return Err(Error::custom(format!(
					"Invalid pack reference format: '{}'. Too many '@' symbols.",
					full_ref
				)));
			}
		};

		// --- Determine name, scope, and sub_path from name_and_path_str
		let (name_and_scope_part, sub_path) = match name_and_path_str.split_once('/') {
			Some((start, path)) => (start, if path.is_empty() { None } else { Some(path.to_string()) }),
			None => (name_and_path_str, None),
		};

		// --- Determine name and scope from name_and_scope_part
		let (name_str, sub_path_scope) = match name_and_scope_part.split_once('$') {
			Some((name_part, scope_part)) => {
				// Ensure name_part is not empty before '$'
				if name_part.is_empty() {
					// Catches cases like "$base", "ns@$base/path"
					return Err(Error::custom(format!(
						"Invalid pack reference format: '{}'. Pack name cannot be empty before '$'.",
						full_ref
					)));
				}

				let scope = match scope_part {
					"base" => PackRefSubPathScope::BaseSupport,
					"workspace" => PackRefSubPathScope::WorkspaceSupport,
					"" => {
						// Handle cases like "name$" or "name$/path"
						return Err(Error::custom(format!(
							"Invalid pack reference scope in '{}'. Scope cannot be empty after '$'. Expected '$base' or '$workspace'",
							full_ref
						)));
					}
					_ => {
						return Err(Error::custom(format!(
							"Invalid pack reference scope in '{}'. Expected '$base' or '$workspace', found '${}'.",
							full_ref, scope_part
						)));
					}
				};
				// Validate name characters before scope
				PackIdentity::validate_name(name_part)?;
				(name_part, scope)
			}
			None => {
				// No scope specified, validate the whole part as name
				PackIdentity::validate_name(name_and_scope_part)?;
				(name_and_scope_part, PackRefSubPathScope::BaseInstalled)
			}
		};

		// --- Validate sub_path: ensure it doesn't contain '$'
		if let Some(ref sp) = sub_path {
			if sp.contains('$') {
				return Err(Error::custom(format!(
					"Invalid pack reference format: '{}'. Character '$' is not allowed in the sub-path.",
					full_ref
				)));
			}
			// Ensure sub_path does not contain ".."
			if sp.split('/').any(|part| part == "..") {
				return Err(Error::custom(format!(
					"Invalid pack reference format: '{}'. Sub-path cannot contain '..'.",
					full_ref
				)));
			}
		}

		// Basic validation: ensure name is not empty after parsing
		// This check is important for cases where name_str is empty after splitting by '/', '$'
		// e.g. "/", "ns@/", "ns@$base/"
		if name_str.is_empty() {
			return Err(Error::custom(format!(
				"Invalid pack reference format: '{}'. Pack name part cannot be empty.",
				full_ref
			)));
		}

		Ok(PackRef {
			namespace: namespace_str.to_string(),
			name: name_str.to_string(),
			sub_path_scope,
			sub_path,
		})
	}
}

/// Implement the Display trait for PartialPackRef
impl std::fmt::Display for PackRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}@{}", self.namespace, self.name)?;

		match self.sub_path_scope {
			PackRefSubPathScope::BaseSupport => write!(f, "$base")?,
			PackRefSubPathScope::WorkspaceSupport => write!(f, "$workspace")?,
			PackRefSubPathScope::BaseInstalled => {} // Do nothing for default
		}

		if let Some(sub_path) = &self.sub_path {
			write!(f, "/{}", sub_path)?;
		}
		Ok(())
	}
}

// endregion: --- PartialPackRef

// region:    --- LocalPackRef

/// This is a Locally Resolved PackRef
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct LocalPackRef {
	pub identity: PackIdentity,
	/// e.g. `text` if `demo@craft/text`
	pub sub_path: Option<String>,
	/// The absolute path of the pack `demo@craft`
	pub pack_dir: SPath,
	pub repo_kind: RepoKind,
}

impl LocalPackRef {
	/// NOTE: Right now ns and pack_name ae in both pack_dir and partial, but that is ok for no
	///       Eventually, need to clean this up.
	pub fn from_partial(pack_dir: PackDir, partial: PackRef) -> Self {
		let repo_kind = pack_dir.repo_kind;
		// Note: This assumes the namespace in pack_dir matches partial.namespace if Some.
		// A check could be added here if needed, but resolution logic should handle this.
		let namespace = partial.namespace;
		let pack_dir_path = pack_dir.path;

		let identity = PackIdentity {
			namespace,
			name: partial.name,
		};

		Self {
			identity,
			sub_path: partial.sub_path,
			pack_dir: pack_dir_path,
			repo_kind,
		}
	}
}

/// Getters
#[allow(unused)]
impl LocalPackRef {
	pub fn identity(&self) -> &PackIdentity {
		&self.identity
	}
	pub fn namespace(&self) -> &str {
		&self.identity.namespace
	}
	pub fn name(&self) -> &str {
		&self.identity.name
	}
	pub fn sub_path(&self) -> Option<&str> {
		self.sub_path.as_deref()
	}
	pub fn pack_dir(&self) -> &SPath {
		&self.pack_dir
	}
	pub fn repo_kind(&self) -> RepoKind {
		self.repo_kind
	}
}

/// Implement the Display trait for PackRef
impl std::fmt::Display for LocalPackRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}@{}", self.namespace(), self.name())?;
		if let Some(sub_path) = &self.sub_path {
			write!(f, "/{}", sub_path)?;
		}
		Ok(())
	}
}

// endregion: --- LocalPackRef

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::assert_contains;

	#[test]
	fn test_pack_pack_ref_from_str_simple() -> Result<()> {
		// -- Setup & Fixtures
		let data = [
			// input, expected_ns, expected_name, expected_scope, expected_sub
			("pro@coder", "pro", "coder", PackRefSubPathScope::BaseInstalled, None),
			(
				"pro@coder/agent.yaml",
				"pro",
				"coder",
				PackRefSubPathScope::BaseInstalled,
				Some("agent.yaml"),
			),
			(
				"pro@coder/", // Trailing slash
				"pro",
				"coder",
				PackRefSubPathScope::BaseInstalled,
				None,
			),
			// With hyphens and underscores
			(
				"my-ns@pack_name-123",
				"my-ns",
				"pack_name-123",
				PackRefSubPathScope::BaseInstalled,
				None,
			),
			(
				"_ns@_name/_sub-path",
				"_ns",
				"_name",
				PackRefSubPathScope::BaseInstalled,
				Some("_sub-path"),
			),
			(
				"_ns@_name$workspace/_sub-path",
				"_ns",
				"_name",
				PackRefSubPathScope::WorkspaceSupport,
				Some("_sub-path"),
			),
		];

		// -- Exec & Check
		for (input, ns, name, scope, sub) in data {
			let pref = PackRef::from_str(input)?;
			assert_eq!(pref.namespace, ns, "Input: {}", input);
			assert_eq!(pref.name, name, "Input: {}", input);
			assert_eq!(pref.sub_path_scope, scope, "Input: {}", input);
			assert_eq!(pref.sub_path.as_deref(), sub, "Input: {}", input);
		}

		Ok(())
	}

	#[test]
	fn test_pack_pack_ref_from_str_with_scope() -> Result<()> {
		// -- Setup & Fixtures
		let data = [
			// input, expected_ns, expected_name, expected_scope, expected_sub
			("pro@coder$base", "pro", "coder", PackRefSubPathScope::BaseSupport, None),
			(
				"pro@coder$base/data.json",
				"pro",
				"coder",
				PackRefSubPathScope::BaseSupport,
				Some("data.json"),
			),
			(
				"pro@coder$workspace",
				"pro",
				"coder",
				PackRefSubPathScope::WorkspaceSupport,
				None,
			),
			(
				"pro@coder$workspace/data.json",
				"pro",
				"coder",
				PackRefSubPathScope::WorkspaceSupport,
				Some("data.json"),
			),
			(
				"pro@coder$base/", // Trailing slash
				"pro",
				"coder",
				PackRefSubPathScope::BaseSupport,
				None,
			),
			// With hyphens/underscores
			(
				"my-ns@pack_name$base/file-1",
				"my-ns",
				"pack_name",
				PackRefSubPathScope::BaseSupport,
				Some("file-1"),
			),
		];

		// -- Exec & Check
		for (input, ns, name, scope, sub) in data {
			let pref = PackRef::from_str(input)?;
			assert_eq!(pref.namespace, ns, "Input: {}", input);
			assert_eq!(pref.name, name, "Input: {}", input);
			assert_eq!(pref.sub_path_scope, scope, "Input: {}", input);
			assert_eq!(pref.sub_path.as_deref(), sub, "Input: {}", input);

			// Check Display roundtrip (where applicable)
			assert_eq!(
				pref.to_string(),
				input.trim_end_matches('/'),
				"Display mismatch for: {}",
				input
			);
		}

		Ok(())
	}

	#[test]
	fn test_pack_pack_ref_from_str_invalids() -> Result<()> {
		// -- Setup & Fixtures
		let data = &[
			// Invalid inputs and expected error message fragments
			("", "No '@' sign"),                                                // Empty string
			("@", "Namespace cannot be empty"),                                 // Just @
			("ns@", "Pack name/path part cannot be empty after '@'."),          // Namespace only
			("@name", "Namespace cannot be empty"),                             // Name only with leading @
			("pro@coder$invalid/data.json", "Invalid pack reference scope"),    // Invalid scope name
			("pro@coder$baseExtra/data.json", "Invalid pack reference scope"),  // Invalid scope name
			("pro@coder$base/data$json", "'$' is not allowed in the sub-path"), // '$' in sub-path
			("pro@coder@", "Too many '@' symbols."),                            // Double @ at the end
			("pro@@coder", "Too many '@' symbols."),                            // Double @ in the middle
			("pro@coder$/sub", "Scope cannot be empty after '$'"),              // Empty scope
			("pro@coder$ /sub", "Invalid pack reference scope"),                // Invalid scope ' '
			("ns@$base", "Pack name cannot be empty before '$'."),              // Empty name before '$'
			("$base", "No '@' sign"),                                           // Just scope, no name
			("/", "No '@' sign"),                                               // Just slash
			("ns@/", "Pack name cannot be empty"),                              // Namespace and slash
			("ns@$base/", "Pack name cannot be empty before '$'."),             // Namespace, scope, and slash
			// Invalid characters
			("n space@coder", "namespace can only contain"),
			("ns@co der", "name can only contain"),
			("n+s@coder", "namespace can only contain"),
			("ns@co+der", "name can only contain"),
			("n$s@coder", "namespace can only contain"), // $ invalid in ns
			// Note: name$scope is valid, but n@ame$invalid is caught by scope check
			("ns@co=der", "name can only contain"),
			// -- For now disalow start with numbers
			("1ns@coder", "namespace can only contain"), // starts with number disallowed by PackIdentity
			("ns@1coder", "name can only contain"),      // starts with number disallowed by PackIdentity
			// -- For now disallow star with ".." (later we will mem-resolve those to allow until it does not go back to minus path)
			("ns@coder/sub/../path", "Sub-path cannot contain '..'"), // '..' in sub-path
			("ns@coder/../sub/path", "Sub-path cannot contain '..'"), // '..' at start of sub-path
			("ns@coder/sub/path/..", "Sub-path cannot contain '..'"), // '..' at end of sub-path
			("ns@coder/..", "Sub-path cannot contain '..'"),          // '..' as sub-path
		];

		// -- Exec & Check
		for (invalid_input, expected_error) in data {
			let result = PackRef::from_str(invalid_input);

			assert!(result.is_err(), "Should fail for invalid input: '{}'", invalid_input);
			if let Err(err) = result {
				assert_contains(&err.to_string(), expected_error);
			} else {
				// This panic will trigger if result is Ok, which shouldn't happen for these inputs
				panic!("Input '{}' should have failed but succeeded.", invalid_input);
			}
		}

		Ok(())
	}
}

// endregion: --- Tests
