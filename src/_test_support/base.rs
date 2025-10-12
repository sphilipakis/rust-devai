use super::Result;
use crate::agent::AgentOptions;
use std::path::Path;

pub const TEST_MODEL: &str = "gpt-5-mini";

pub const SANDBOX_01_WKS_DIR: &str = "./tests-data/sandbox-01";

pub const SANDBOX_01_BASE_AIPACK_DIR: &str = "./tests-data/.aipack-base";

#[allow(unused)]
pub const TESTS_DATA_DIR: &str = "./tests-data";

#[allow(unused)]
pub const TESTS_TMP_DIR: &str = "./tests-data/tmp";

pub fn default_agent_config_for_test() -> AgentOptions {
	AgentOptions::new(TEST_MODEL)
}

#[allow(unused)]
pub fn read_test_file(rel_test_path: impl AsRef<Path>) -> Result<String> {
	let path = std::path::Path::new(TESTS_DATA_DIR).join(rel_test_path);
	let content = std::fs::read_to_string(path)?;
	Ok(content)
}
