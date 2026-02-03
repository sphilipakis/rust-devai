use crate::agent::AgentOptions;
use crate::script::{LuaValueExt as _, lua_value_to_serde_value};
use mlua::{FromLua, Lua, Value};

/// Options for the `aip.agent.run(name, options)` function, including inputs and agent option overrides.
#[derive(Debug, Default)]
pub struct RunAgentOptions {
	pub inputs: Option<Vec<serde_json::Value>>,
	pub options: Option<AgentOptions>,
}

impl FromLua for RunAgentOptions {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(Self::default()),
			Value::Table(table) => {
				let inputs = table.x_get_value("inputs").map(lua_value_to_serde_value).transpose()?;

				let inputs = match inputs {
					Some(serde_json::Value::Array(values)) => Some(values),
					Some(_) => {
						return Err(mlua::Error::FromLuaConversionError {
							from: "Table",
							to: "RunAgentOptions".to_string(),
							message: Some("The 'inputs' field must be a Lua array".into()),
						});
					}
					None => None,
				};

				let options = table
					.x_get_value("options")
					.map(|o| AgentOptions::from_lua(o, lua))
					.transpose()?;

				Ok(Self { inputs, options })
			}
			other => Err(mlua::Error::FromLuaConversionError {
				from: other.type_name(),
				to: "RunAgentOptions".to_string(),
				message: Some("RunAgentOptions argument must be nil or a table { inputs?, options? }".into()),
			}),
		}
	}
}
