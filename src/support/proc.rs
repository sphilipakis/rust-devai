use crate::{Error, Result};
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Default)]
pub struct ProcOptions {
	pub cwd: Option<String>,
	#[cfg(windows)]
	pub creation_flags: Option<u32>,
}

impl ProcOptions {
	pub fn with_cwd(self, cwd: impl Into<String>) -> Self {
		#[allow(clippy::needless_update)] // must do for windows
		Self {
			cwd: Some(cwd.into()),
			..self
		}
	}
}

fn apply_options(command: &mut Command, options: Option<&ProcOptions>) {
	#[allow(clippy::collapsible_if)]
	if let Some(opts) = options {
		if let Some(cwd) = &opts.cwd {
			command.current_dir(cwd);
		}
		#[cfg(windows)]
		if let Some(flags) = opts.creation_flags {
			command.creation_flags(flags);
		}
	}
}

fn format_command(cmd: &str, args: &[&str]) -> String {
	if args.is_empty() {
		cmd.to_string()
	} else {
		format!("{cmd} {}", args.join(" "))
	}
}

pub async fn proc_exec_to_output(cmd: &str, args: &[&str], options: Option<&ProcOptions>) -> Result<String> {
	let mut command = Command::new(cmd);
	command.args(args);
	command.stdout(Stdio::piped());
	command.stderr(Stdio::piped());
	apply_options(&mut command, options);

	let command_repr = format_command(cmd, args);

	let output = command
		.output()
		.await
		.map_err(|err| Error::custom(format!("Failed to execute '{command_repr}'. Cause: {err}")))?;

	if !output.status.success() {
		let status_desc = match output.status.code() {
			Some(code) => format!("exit code {code}"),
			None => "terminated by signal".to_string(),
		};
		let stderr = String::from_utf8_lossy(&output.stderr);
		return Err(Error::custom(format!(
			"Command '{command_repr}' failed with {status_desc}. Stderr: {stderr}"
		)));
	}

	let stdout = String::from_utf8(output.stdout).map_err(|err| {
		Error::custom(format!(
			"Failed to decode stdout for '{command_repr}' as UTF-8. Cause: {err}"
		))
	})?;

	Ok(stdout)
}

pub async fn proc_exec(cmd: &str, args: &[&str], options: Option<&ProcOptions>) -> Result<()> {
	let mut command = Command::new(cmd);
	command.args(args);
	command.stdin(Stdio::inherit());
	command.stdout(Stdio::inherit());
	command.stderr(Stdio::inherit());
	apply_options(&mut command, options);

	let command_repr = format_command(cmd, args);

	let status = command
		.status()
		.await
		.map_err(|err| Error::custom(format!("Failed to execute '{command_repr}'. Cause: {err}")))?;

	if !status.success() {
		let status_desc = match status.code() {
			Some(code) => format!("exit code {code}"),
			None => "terminated by signal".to_string(),
		};
		return Err(Error::custom(format!(
			"Command '{command_repr}' failed with {status_desc}."
		)));
	}

	Ok(())
}
