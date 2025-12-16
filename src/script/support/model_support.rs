//! Model-related support utilities for Lua script modules.
//!
//! This module provides shared functionality for `aip_run`, `aip_task`, and `aip_pin` modules
//! that need to interact with the model layer.

use crate::Result;
use crate::model::{PinBmc, PinForRunSave, PinForTaskSave, RuntimeCtx};
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::types::uc;
use mlua::{FromLua, Lua, Value, Variadic};

// region:    --- Pin Support

/// Shared implementation for both `run.pin` and `task.pin`.
pub fn create_pin(lua: &Lua, runtime: &Runtime, for_task: bool, args: Variadic<Value>) -> Result<()> {
	let cmd = PinCommand::from_lua_variadic(lua, args)?;

	let ctx = RuntimeCtx::extract_from_global(lua)?;

	let mm = runtime.mm();
	let (run_id, task_id) = {
		let run_id = ctx.get_run_id(mm)?.ok_or("Cannot create pin – no RUN context available")?;
		let task_id = if for_task { ctx.get_task_id(mm)? } else { None };
		(run_id, task_id)
	};

	if for_task {
		let task_id = task_id.ok_or(
			"Cannot call 'aip.task.pin(...)' in a before all or after all code block.\nCall `aip.run.pin(..)`'",
		)?;
		let pin_c = PinForTaskSave {
			run_id,
			task_id,
			iden: cmd.iden,
			priority: cmd.priority,
			content: Some(cmd.content),
		};

		PinBmc::save_task_pin(mm, pin_c)?
	} else {
		let pin_c = PinForRunSave {
			run_id,
			iden: cmd.iden,
			priority: cmd.priority,
			content: Some(cmd.content),
		};

		PinBmc::save_run_pin(mm, pin_c)?
	};

	Ok(())
}

// -- PinCommand
// Captures the parsed arguments provided to the Lua `...pin(..)` helpers.
struct PinCommand {
	iden: String,
	priority: Option<f64>,
	content: String,
}

impl PinCommand {
	/// Parses the variadic Lua arguments for the two supported signatures:
	///
	/// 1. `pin(iden, priority, content)`
	/// 2. `pin(iden, content)`
	///
	/// `content` can be a string (or convertible primitive value) or a structured table
	/// `{label?: string, content: string}` (mapped to uc::Marker).
	///
	/// Returns an informative error if the arguments do not match either form.
	fn from_lua_variadic(lua: &Lua, args: Variadic<Value>) -> Result<Self> {
		match args.len() {
			2 => {
				let mut args = args.into_iter();
				let iden = args
					.next()
					.ok_or("aip...pin(iden, content) – expected <string> for parameter `iden`.")?;
				let iden = iden
					.x_as_lua_str()
					.ok_or("aip...pin(iden, content) – expected <string> for parameter `iden`.")?;

				let content = args.next().ok_or("aip...pin(iden, content) – expected content.")?;
				let content = Self::value_to_uc_string(lua, content)?;

				Ok(Self {
					iden: iden.to_string(),
					priority: None,
					content,
				})
			}
			3 => {
				let mut args = args.into_iter();
				let iden = args
					.next()
					.ok_or("aip...pin(iden, content) – expected <string> for parameter `iden`.")?;
				let iden = iden
					.x_as_lua_str()
					.ok_or("aip...pin(iden, content) – expected <string> for parameter `iden`.")?;

				let priority = args
					.next()
					.and_then(|v| v.x_as_f64())
					.ok_or("aip...pin(iden, priority, content) – expected <number> for parameter `priority`.")?;

				let content = args.next().ok_or("aip...pin(iden, content) – expected content.")?;
				let content = Self::value_to_uc_string(lua, content)?;

				Ok(Self {
					iden: iden.to_string(),
					priority: Some(priority),
					content,
				})
			}
			_ => Err(crate::Error::custom(
				"aip...pin(...) – expected 2 or 3 parameters: (iden, content) or (iden, priority, content).",
			)),
		}
	}

	/// Convert into a UC Component
	/// For now, only support uc::Marker
	fn value_to_uc_string(lua: &Lua, val: Value) -> Result<String> {
		let uc_comp = uc::Marker::from_lua(val, lua)?;
		let json_string = serde_json::to_string_pretty(&uc_comp)
			.map_err(|err| crate::Error::cc("Cannot seralize uc component", err))?;

		Ok(json_string)
	}
}

// endregion: --- Pin Support
