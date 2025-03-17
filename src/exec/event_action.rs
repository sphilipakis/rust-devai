//! The executor command
//! Note: For now, the content of the variant of the ExecCommand often contain the CliArgs,
//!       but this will eventual change to have it's own

use crate::cli::{InitArgs, InstallArgs, ListArgs, NewArgs, PackArgs, RunArgs};
use crate::exec::RunAgentParams;

/// Executor Action Event that needs to be performed
///
/// When a system part needs to perform an action, it should send one of these events.
///
/// For now, there are split in 3 categories
/// - The cli commands
/// - The interactive commands (when pressing in the "cli interface")
/// - The agent commands (when Lua is asking to execute an agent agent)
///
/// NOTE: This is not the `ExecStateEvent` which is sent to the hub.
#[derive(Debug)]
pub enum ExecActionEvent {
	// -- CLI Commands
	/// This will init the workspace with `.aipack/`
	/// and the base with `~/.aipack-base`
	CmdInit(InitArgs),
	/// This will init only the base
	CmdInitBase,
	/// This is the result of a CLI run
	CmdRun(RunArgs),
	CmdList(ListArgs),
	CmdPack(PackArgs),
	CmdInstall(InstallArgs),

	// -- Interactive Commands
	Redo,
	OpenAgent,

	// -- Agent Commands
	RunAgent(RunAgentParams),

	// -- To be deprecated or redesigned
	/// Eventually will get deprecated
	#[allow(unused)]
	CmdNewAgent(NewArgs),
}
