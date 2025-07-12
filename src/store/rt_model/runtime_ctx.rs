use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::store::base::DbBmc as _;
use crate::store::rt_model::{RunBmc, TaskBmc};
use crate::store::{Id, ModelManager};
use crate::store::{Result, Stage};
use mlua::FromLua;
use uuid::Uuid;

/// The Lua Context need to track from where the data, event are sent.
///
/// NOTE: Right now, just support run & task uid, but should probably support stage as
///
/// NOTE: Might want to move it to runtime/ module (not sure)
#[derive(Debug, Clone, Default)]
pub struct RuntimeCtx {
	run_uid: Option<Uuid>,
	task_uid: Option<Uuid>,
	stage: Option<Stage>,
}

#[allow(unused)]
impl RuntimeCtx {
	pub fn run_uid(&self) -> Option<Uuid> {
		self.run_uid
	}

	pub fn task_uid(&self) -> Option<Uuid> {
		self.task_uid
	}

	pub fn stage(&self) -> Option<Stage> {
		self.stage
	}
}

/// Builder with_, ...
impl RuntimeCtx {
	pub fn with_stage(&self, stage: Stage) -> Self {
		let mut ctx = self.clone();
		ctx.stage = Some(stage);
		ctx
	}
}

// region:    --- Model Related

impl RuntimeCtx {
	pub fn from_run_id(runtime: &Runtime, run_id: Id) -> Result<Self> {
		let mm = runtime.mm();
		let run_uid = RunBmc::get_uid(mm, run_id)?;
		Ok(RuntimeCtx {
			run_uid: Some(run_uid),
			task_uid: None,
			stage: None,
		})
	}

	pub fn from_run_task_ids(runtime: &Runtime, run_id: Option<Id>, task_id: Option<Id>) -> Result<Self> {
		let mm = runtime.mm();
		let run_uid = run_id.map(|run_id| RunBmc::get_uid(mm, run_id)).transpose()?;
		let task_uid = task_id.map(|task_id| TaskBmc::get_uid(mm, task_id)).transpose()?;
		Ok(RuntimeCtx {
			run_uid,
			task_uid,
			stage: None,
		})
	}

	pub fn get_run_id(&self, mm: &ModelManager) -> Result<Option<Id>> {
		let id = self.run_uid.map(|v| RunBmc::get_id_for_uid(mm, v)).transpose()?;
		Ok(id)
	}

	pub fn get_task_id(&self, mm: &ModelManager) -> Result<Option<Id>> {
		let id = self.task_uid.map(|v| TaskBmc::get_id_for_uid(mm, v)).transpose()?;
		Ok(id)
	}
}

// endregion: --- Model Related

// region:    --- Lua Related

impl RuntimeCtx {
	// NOTE: for now, use the crate::Result for this one, since called in lua context
	pub fn extract_from_global(lua: &mlua::Lua) -> crate::Result<Self> {
		let globals = lua.globals();
		if let Some(ctx) = globals.x_get_value("CTX") {
			Ok(RuntimeCtx::from_lua(ctx, lua)?)
		} else {
			Ok(RuntimeCtx::default())
		}
	}
}

impl FromLua for RuntimeCtx {
	fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
		let run_uid = value.x_get_string("RUN_UID").and_then(|s| Uuid::parse_str(&s).ok());
		let task_uid = value.x_get_string("TASK_UID").and_then(|s| Uuid::parse_str(&s).ok());
		let stage = value.x_get_string("STAGE").and_then(|s| Stage::from_str(&s));

		Ok(RuntimeCtx {
			run_uid,
			task_uid,
			stage,
		})
	}
}

// endregion: --- Lua Related
