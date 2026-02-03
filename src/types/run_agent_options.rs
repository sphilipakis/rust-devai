use crate::agent::AgentOptions;
use crate::script::{LuaValueExt as _, lua_value_to_serde_value};
use mlua::{FromLua, Lua, Value};
use simple_fs::SPath;

/// Options for the `aip.agent.run(name, options)` function, including inputs and agent option overrides.
///
/// NOTE: In Lua, this supports both `.input` (single) and `.inputs` (array), which are merged
///       into the `inputs` field here.
#[derive(Debug, Default)]
pub struct RunAgentOptions {
	pub inputs: Option<Vec<serde_json::Value>>,
	pub options: Option<AgentOptions>,
	pub agent_base_dir: Option<SPath>,
}

impl FromLua for RunAgentOptions {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(Self::default()),
			Value::Table(table) => {
				// -- input (single value, since 0.8.15)
				let input = table.x_get_value("input").map(lua_value_to_serde_value).transpose()?;

				// -- inputs (array of values)
				let inputs_raw = table.x_get_value("inputs").map(lua_value_to_serde_value).transpose()?;

				// Validate inputs_raw is an array if present, and merge with input
				let inputs_vec = match inputs_raw {
					Some(serde_json::Value::Array(mut values)) => {
						if let Some(input_val) = input {
							let mut vec = Vec::with_capacity(values.len() + 1);
							vec.push(input_val);
							vec.append(&mut values);
							vec
						} else {
							values
						}
					}
					Some(_) => {
						return Err(mlua::Error::FromLuaConversionError {
							from: "Table",
							to: "RunAgentOptions".to_string(),
							message: Some("The 'inputs' field must be a Lua array".into()),
						});
					}
					None => input.map(|v| vec![v]).unwrap_or_default(),
				};

				let inputs = if inputs_vec.is_empty() { None } else { Some(inputs_vec) };

				// -- options
				let options = table
					.x_get_value("options")
					.map(|o| AgentOptions::from_lua(o, lua))
					.transpose()?;

				// -- agent_base_dir
				let agent_base_dir = table.x_get_string("agent_base_dir").map(SPath::new);

				Ok(Self {
					inputs,
					options,
					agent_base_dir,
				})
			}
			other => Err(mlua::Error::FromLuaConversionError {
				from: other.type_name(),
				to: "RunAgentOptions".to_string(),
				message: Some("RunAgentOptions argument must be nil or a table { inputs?, options? }".into()),
			}),
		}
	}
}
