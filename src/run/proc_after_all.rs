//! The before all processor

use crate::Result;
use crate::agent::Agent;
use crate::run::Literals;
use crate::runtime::Runtime;
use crate::store::rt_model::RuntimeCtx;
use crate::store::{Id, Stage};
use serde_json::Value;

// region:    --- Types

pub struct ProcAfterAllResponse {
	pub after_all: Option<Value>,
	pub outputs: Option<Vec<Value>>,
}

// endregion: --- Types

#[allow(clippy::too_many_arguments)]
pub async fn process_after_all(
	runtime: &Runtime,
	base_rt_ctx: RuntimeCtx,
	run_id: Id,
	agent: &Agent,
	literals: Literals,
	before_all: Value,
	inputs: Vec<Value>,
	outputs: Option<Vec<Value>>,
) -> Result<ProcAfterAllResponse> {
	if let Some(after_all_script) = agent.after_all_script() {
		let outputs_value = if let Some(outputs) = outputs.as_ref() {
			Value::Array(outputs.clone())
		} else {
			Value::Null
		};

		let lua_engine = runtime.new_lua_engine_with_ctx(&literals, base_rt_ctx.with_stage(Stage::AfterAll))?;
		let lua_scope = lua_engine.create_table()?;
		let inputs = Value::Array(inputs);
		lua_scope.set("inputs", lua_engine.serde_to_lua_value(inputs)?)?;
		// Will be Value::Null if outputs were not collected
		lua_scope.set("outputs", lua_engine.serde_to_lua_value(outputs_value)?)?;
		lua_scope.set("before_all", lua_engine.serde_to_lua_value(before_all)?)?;
		lua_scope.set("options", agent.options_as_ref())?;

		// -- Rt Step - After All Start
		runtime.step_aa_start(run_id).await?;

		let lua_value = lua_engine.eval(after_all_script, Some(lua_scope), Some(&[agent.file_dir()?.as_str()]))?;

		// -- Rt Step - After All End
		runtime.step_aa_end(run_id).await?;

		let after_all = Some(serde_json::to_value(lua_value)?);

		Ok(ProcAfterAllResponse { after_all, outputs })
	} else {
		Ok(ProcAfterAllResponse {
			after_all: None,
			outputs,
		})
	}
}
