use crate::Result;
use crate::agent::{Agent, AgentRef};
use crate::dir_context::join_support_pack_ref;
use crate::runtime::Runtime;
use crate::script::LuaEngine;
use std::sync::Arc;

/// TODO: Will need to put the Vec in Arc, since this clone what a bit
#[derive(Debug, Default, Clone)]
pub struct Literals {
	/// The store of all literals, pattern and value
	/// e.g. `vec![("&AIPACK_AGENT_DIR","./.aipack/custom/command-agent/some.aipack")]`
	store: Arc<Vec<(&'static str, String)>>,
}

/// Constructors
impl Literals {
	pub(super) fn from_runtime_and_agent_path(runtime: &Runtime, agent: &Agent) -> Result<Literals> {
		// let mut literals = Literals::default();
		let dir_context = runtime.dir_context();

		let mut store = Vec::new();

		let agent_path = dir_context.current_dir().join(agent.file_path());

		let agent_dir = agent_path
			.parent()
			.ok_or_else(|| format!("Agent {agent_path} does not have a parent dir"))?;

		let aipack_paths = dir_context.aipack_paths();

		store.push(("PWD", dir_context.current_dir().to_string()));

		let session_str = runtime.session_str();

		store.push(("SESSION_UID", session_str.to_string()));

		if let Some(tmp_dir) = aipack_paths.tmp_dir(runtime.session()) {
			store.push(("TMP_DIR", tmp_dir.to_string()))
		}

		// -- AIPACK information
		store.push(("AIPACK_VERSION", crate::VERSION.to_string()));

		// -- Pack Identity / Ref
		if let AgentRef::PackRef(pack_ref) = agent.agent_ref() {
			store.push(("PACK_NAMESPACE", pack_ref.namespace().to_string()));
			store.push(("PACK_NAME", pack_ref.name().to_string()));
			if let Some(sub_path) = pack_ref.sub_path.as_deref() {
				store.push(("PACK_SUB_PATH", sub_path.to_string()));
			}
			// This will be `demo@craft/some/path`
			store.push(("PACK_REF", pack_ref.to_string()));
			// This will be `demo@craft`
			store.push(("PACK_IDENTITY", pack_ref.identity().to_string()));

			// This will be the absolute path of `demo@craft`
			store.push(("PACK_DIR", pack_ref.pack_dir.to_string()));

			let identity_path = &pack_ref.identity().identity_as_path();

			if let Some(aipack_wks_dir) = dir_context.aipack_paths().aipack_wks_dir() {
				let path = join_support_pack_ref(aipack_wks_dir.path(), identity_path).to_string();
				store.push(("PACK_WORKSPACE_SUPPORT_DIR", path.to_string()));
			}

			let pack_support_base = join_support_pack_ref(aipack_paths.aipack_base_dir().path(), identity_path);
			store.push(("PACK_BASE_SUPPORT_DIR", pack_support_base.to_string()));
		}

		// -- Workspace / base dirs
		// NOTE: We have to think about support the lack of workspace_dir
		if let Some(wks_dir) = dir_context.wks_dir() {
			store.push(("WORKSPACE_DIR", wks_dir.to_string()));
		}
		// Those are the absolute path for  and `.aipack/`
		if let Some(aipack_wks_dir) = aipack_paths.aipack_wks_dir() {
			store.push(("WORKSPACE_AIPACK_DIR", aipack_wks_dir.to_string()));
		}
		// `~/.aipack-base/`
		store.push(("BASE_AIPACK_DIR", aipack_paths.aipack_base_dir().to_string()));

		// -- Agent Information
		store.push(("AGENT_NAME", agent.name().to_string()));
		store.push(("AGENT_FILE_NAME", agent_path.name().to_string()));
		store.push(("AGENT_FILE_PATH", agent_path.as_str().to_string()));
		store.push(("AGENT_FILE_DIR", agent_dir.to_string()));
		store.push(("AGENT_FILE_STEM", agent_path.stem().to_string()));

		Ok(Self { store: Arc::new(store) })
	}
}

/// Getters
impl Literals {
	// pub fn append(&mut self, pattern: impl Into<String>, value: impl Into<String>) {

	// }

	// Your existing add method...
	#[allow(unused)]
	pub fn as_strs(&self) -> Vec<(&str, &str)> {
		self.store.iter().map(|(p, v)| (*p, v.as_str())).collect()
	}
}

/// Transformers
impl Literals {
	/// Generate a Lua Value
	/// Note: Similar to into_lua but with no
	pub fn to_lua(&self, lua_engine: &LuaEngine) -> Result<mlua::Value> {
		let table = lua_engine.create_table()?;
		for (name, value) in self.as_strs() {
			table.set(name, value)?;
		}
		Ok(mlua::Value::Table(table))
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, assert_ends_with, run_reflective_agent};
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_run_literals_aipack_dir() -> Result<()> {
		let script = r#"
return {
		TMP_DIR                     = CTX.TMP_DIR,
		SESSION_UID                 = CTX.SESSION_UID,
	  WORKSPACE_DIR               = CTX.WORKSPACE_DIR,
		WORKSPACE_AIPACK_DIR        = CTX.WORKSPACE_AIPACK_DIR,
		BASE_AIPACK_DIR             = CTX.BASE_AIPACK_DIR,
		AGENT_FILE_NAME             = CTX.AGENT_FILE_NAME,
		AGENT_FILE_PATH             = CTX.AGENT_FILE_PATH,
		AGENT_FILE_DIR              = CTX.AGENT_FILE_DIR,
		AGENT_FILE_STEM             = CTX.AGENT_FILE_STEM,
		-- those should be absent in the json, because nul in this case
		PACK_BASE_SUPPORT_DIR       = CTX.PACK_BASE_SUPPORT_DIR,
		PACK_WORKSPACE_SUPPORT_DIR  = CTX.PACK_WORKSPACE_SUPPORT_DIR,
}
		"#;

		// -- Exec
		let res = run_reflective_agent(script, None).await?;

		// -- Check
		// check session
		let session = res.x_get_str("SESSION_UID")?;
		assert_eq!(session.len(), 36);
		assert_contains(session, "-7"); // v7
		// check tmp_dir
		let tmp_dir = res.x_get_str("TMP_DIR")?;
		assert_ends_with(tmp_dir, &format!(".aipack/.session/{session}/tmp"));
		// check workspace
		assert_ends_with(res.x_get_str("WORKSPACE_DIR")?, "tests-data/sandbox-01");
		assert_ends_with(res.x_get_str("WORKSPACE_AIPACK_DIR")?, "tests-data/sandbox-01/.aipack");
		// check agent
		assert_eq!(res.x_get_str("AGENT_FILE_NAME")?, "mock-reflective-agent.aip");
		assert_eq!(res.x_get_str("AGENT_FILE_STEM")?, "mock-reflective-agent");
		assert_ends_with(res.x_get_str("AGENT_FILE_PATH")?, "mock-reflective-agent.aip");
		// check base
		assert_ends_with(res.x_get_str("BASE_AIPACK_DIR")?, "tests-data/.aipack-base");

		Ok(())
	}
}

// endregion: --- Tests
