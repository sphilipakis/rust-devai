use crate::agent::AgentOptions;

pub const TEST_MODEL: &str = "gpt-5-mini";

pub const SANDBOX_01_WKS_DIR: &str = "./tests-data/sandbox-01";

pub const SANDBOX_01_BASE_AIPACK_DIR: &str = "./tests-data/.aipack-base";

#[allow(unused)]
pub const TESTS_TMP_DIR: &str = "./tests-data/tmp";

pub fn default_agent_config_for_test() -> AgentOptions {
	AgentOptions::new(TEST_MODEL)
}
