use crate::model::base::DbBmc as _;
use crate::model::{Id, ModelManager, Result, RunBmc, Stage, TaskBmc};
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
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
	run_num: Option<i32>,
	#[allow(unused)]
	parent_run_uid: Option<Uuid>,
	task_uid: Option<Uuid>,
	task_num: Option<i32>,
	stage: Option<Stage>,
}

#[allow(unused)]
impl RuntimeCtx {
	pub fn run_uid(&self) -> Option<Uuid> {
		self.run_uid
	}

	pub fn run_num(&self) -> Option<i32> {
		self.run_num
	}

	pub fn parent_run_uid(&self) -> Option<Uuid> {
		self.parent_run_uid
	}

	pub fn task_uid(&self) -> Option<Uuid> {
		self.task_uid
	}

	pub fn task_num(&self) -> Option<i32> {
		self.task_num
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
		let run_uids = RunBmc::get_uids(mm, run_id)?;
		Ok(RuntimeCtx {
			run_uid: Some(run_uids.uid),
			run_num: Some(run_uids.id.as_i64() as i32), // For now, use db id as the num
			parent_run_uid: run_uids.parent_uid,
			task_uid: None,
			task_num: None,
			stage: None,
		})
	}

	pub fn from_run_task_ids(runtime: &Runtime, run_id: Option<Id>, task_id: Option<Id>) -> Result<Self> {
		let mut runtime_ctx = if let Some(run_id) = run_id {
			RuntimeCtx::from_run_id(runtime, run_id)?
		} else {
			RuntimeCtx::default()
		};

		if let Some(task_id) = task_id {
			let mm = runtime.mm();
			let ids = TaskBmc::get_ids(mm, task_id)?;
			runtime_ctx.task_uid = Some(ids.uid);
			runtime_ctx.task_num = Some(ids.idx + 1); // because index start at 0
		};

		Ok(runtime_ctx)
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
		let parent_run_uid = value.x_get_string("PARENT_RUN_UID").and_then(|s| Uuid::parse_str(&s).ok());
		let task_uid = value.x_get_string("TASK_UID").and_then(|s| Uuid::parse_str(&s).ok());
		let stage = value.x_get_string("STAGE").and_then(|s| Stage::from_str(&s));
		let run_num = value.x_get_i64("RUN_NUM").map(|v| v as i32);
		let task_num = value.x_get_i64("TASK_NUM").map(|v| v as i32);

		Ok(RuntimeCtx {
			run_uid,
			run_num,
			parent_run_uid,
			task_uid,
			task_num,
			stage,
		})
	}
}

// endregion: --- Lua Related
