//! Defines the `cmd` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `cmd` module exposes functions to execute system commands.
//!
//! ### Functions
//!
//! - `aip.cmd.exec(cmd_name: string, args?: string | list): {stdout: string, stderr: string, exit: number}`

use crate::Result;
use crate::runtime::Runtime;
use crate::script::lua_script::helpers::to_vec_of_strings;
use mlua::{Lua, Table, Value};
use std::process::Command;

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let exec_fn = lua.create_function(cmd_exec)?;

	table.set("exec", exec_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Execute a system command with optional arguments.
///
/// ```lua
/// -- API Signature
/// aip.cmd.exec(cmd_name: string, args?: string | list): CmdResponse
/// ```
///
/// The command will be executed using the system shell. Arguments can be provided as a single string
/// or a table of strings.
///
/// Note: To maximize compatiblity on windows, the command will be wrapped with `cmd /C cmd_name args..`
///
/// ### Example
///
/// ```lua
/// -- Single string argument
/// local result = aip.cmd.exec("echo", "hello world")
///
/// -- Table of arguments
/// local result = aip.cmd.exec("ls", {"-l", "-a"})
/// ```
///
/// ### Return (CmdResponse)
///
/// Returns when the command executes successfully (exit code 0).
///
/// ```ts
/// {
///   stdout: string,  // Standard output from the command
///   stderr: string,  // Standard error from the command
///   exit:   number   // Exit code (0 for success)
/// }
/// ```
///
/// ### Error
///
/// ```ts
/// {
///   stdout: string | nil,  // Standard output if available
///   stderr: string | nil,  // Standard error if available
///   exit:   number | nil,  // Exit code if available
///   error : string         // Error message from command execution
/// }
/// ```
fn cmd_exec(lua: &Lua, (cmd_name, args): (String, Option<Value>)) -> mlua::Result<Value> {
	let args = args.map(|args| to_vec_of_strings(args, "command args")).transpose()?;

	let mut command = cross_command(&cmd_name, args)?;

	match command.output() {
		Ok(output) => {
			let stdout = String::from_utf8_lossy(&output.stdout).to_string();
			let stderr = String::from_utf8_lossy(&output.stderr).to_string();
			let exit_code = output.status.code().unwrap_or(-1) as i64;

			let res = lua.create_table()?;
			res.set("stdout", stdout.as_str())?;
			res.set("stderr", stderr.as_str())?;
			res.set("exit", exit_code)?;

			if exit_code == 0 {
				Ok(Value::Table(res))
			} else {
				res.set("error", format!("Command exited with non-zero status: {}", exit_code))?;
				let cmd = command.get_program().to_str().unwrap_or_default();
				let args = command
					.get_args()
					.map(|a| a.to_str().unwrap_or_default())
					.collect::<Vec<&str>>();
				let args = args.join(" ");
				Err(crate::Error::custom(format!(
					"\
Fail to execute: {cmd} {args}
stdout:\n{stdout}\n
stderr:\n{stderr}\n
exit code: {exit_code}\n"
				))
				.into())
			}
		}
		Err(err) => {
			let cmd = command.get_program().to_str().unwrap_or_default();
			let args = command
				.get_args()
				.map(|a| a.to_str().unwrap_or_default())
				.collect::<Vec<&str>>();
			let args = args.join(" ");
			Err(crate::Error::custom(format!(
				"\
Fail to execute: {cmd} {args}
Cause:\n{err}"
			))
			.into())
		}
	}
}

// region:    --- Support

/// Create a command, and make it a `cmd /C cmd_name args..` for windows
fn cross_command(cmd_name: &str, args: Option<Vec<String>>) -> Result<Command> {
	let command = if cfg!(windows) {
		let full_cmd = if let Some(args) = args {
			// Quote arguments if needed and join
			let joined = args.join(" ");
			format!("{cmd_name} {joined}")
		} else {
			cmd_name.to_string()
		};

		let mut cmd = Command::new("cmd");
		cmd.args(["/C", &full_cmd]);
		cmd
	} else {
		let mut cmd = Command::new(cmd_name);
		if let Some(args) = args {
			cmd.args(args);
		}

		cmd
	};

	Ok(command)
}
// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::lua_script::aip_cmd;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_cmd_exec_echo_single_arg() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd")?;
		let script = r#"
			return aip.cmd.exec("echo", "hello world")
		"#;
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.x_get_str("stdout")?.trim(), "hello world");
		assert_eq!(res.x_get_str("stderr")?, "");
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_echo_multiple_args() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd")?;
		let script = r#"
			return aip.cmd.exec("echo", {"hello", "world"})
		"#;
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.x_get_str("stdout")?.trim(), "hello world");
		assert_eq!(res.x_get_str("stderr")?, "");
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_invalid_command_pcall() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd")?;
		let script = r#"
			local ok, err = pcall(function()
				return aip.cmd.exec("nonexistentcommand")
			end)
			return err -- to trigger the error on the rust side
		"#;
		let Err(err) = eval_lua(&lua, script) else {
			return Err("Should have returned an error".into());
		};

		// -- Check
		let err = err.to_string();
		assert_contains(&err, "nonexistentcommand");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_invalid_command_direct() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd")?;
		let script = r#"return aip.cmd.exec("nonexistentcommand")"#;

		let Err(err) = eval_lua(&lua, script) else {
			return Err("Should have returned an error".into());
		};

		// -- Check
		let err = err.to_string();
		assert_contains(&err, "nonexistentcommand");

		Ok(())
	}
}

// endregion: --- Tests
