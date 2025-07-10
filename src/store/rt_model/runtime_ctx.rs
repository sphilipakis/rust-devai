use crate::script::LuaValueExt;
use crate::store::Result;
use crate::store::base::DbBmc as _;
use crate::store::rt_model::{RunBmc, TaskBmc};
use crate::store::{Id, ModelManager};
use mlua::FromLua;
use uuid::Uuid;

/// The Lua Context need to track from where the data, event are sent.
///
/// NOTE: Right now, just support run & task uid, but should probably support stage as
///
/// NOTE: This struct is exposed in the store/rt-model, so, probably need to move there
///       in some new name like RuntimeCtx perhaps
#[derive(Debug, Clone, Default)]
pub struct RuntimeCtx {
	run_uid: Option<Uuid>,
	task_uid: Option<Uuid>,
}

#[allow(unused)]
impl RuntimeCtx {
	pub fn run_uid(&self) -> Option<Uuid> {
		self.run_uid
	}

	pub fn task_uid(&self) -> Option<Uuid> {
		self.task_uid
	}
}

// region:    --- Model Related

impl RuntimeCtx {
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
		Ok(RuntimeCtx { run_uid, task_uid })
	}
}

// endregion: --- Lua Related
