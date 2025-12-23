# AIPack Executor Specification for LLM

This document provides a concise reference for the AIPack `exec` module architecture, command events, and lifecycle management.

## Core Architecture

The `exec` module follows a decoupled command pattern. The `Executor` runs an asynchronous loop that processes `ExecActionEvent` variants.

### Executor & ExecutorTx (src/exec/executor.rs)

- **Executor**: Owns the receiver and manages the active task count. It spawns a new Tokio task for every action received.
- **ExecutorTx**: A cloneable sender wrapper.
    - `send(ExecActionEvent)`: Async send.
    - `send_sync(ExecActionEvent)`: Synchronous send (flume).
    - `send_sync_spawn_and_block(ExecActionEvent)`: Used when calling from a sync context where the event must be processed in parallel (e.g., Lua module calls).

## Event Types

### Action Events (src/exec/event_action.rs)

`ExecActionEvent` represents "Input" into the executor.

```rust
pub enum ExecActionEvent {
    // -- CLI Commands (Directly from CliArgs)
    CmdInit(InitArgs),
    CmdInitBase,
    CmdList(ListArgs),
    CmdPack(PackArgs),
    CmdInstall(InstallArgs),
    CmdCheckKeys(CheckKeysArgs),
    CmdXelfSetup(XelfSetupArgs),
    CmdXelfUpdate(XelfUpdateArgs),

    // -- Interactive/UI Commands
    OpenAgent,
    Redo,
    CancelRun,

    // -- Agent Logic Commands
    RunSubAgent(RunSubAgentParams),
    CmdRun(RunArgs),
}
```

### Status Events (src/exec/event_status.rs)

`ExecStatusEvent` represents lifecycle "Output" published to the `Hub`.

- `StartExec` / `EndExec`: Fired when the executor starts/finishes a batch of actions (tracked by `active_actions` counter).
- `RunStart` / `RunEnd`: Fired specifically for Agent Run/Redo cycles.

## Submodule Responsibilities

### Initialization (src/exec/init/...)

Handles the setup of the local environment.
- `init_base(force)`: Ensures `~/.aipack-base/` exists, updates default configs, and extracts built-in packs.
- `init_wks(path, show_info)`: Ensures `.aipack/` exists in the project, creates `config.toml`.
- `assets.rs`: Logic for extracting files from the embedded `ASSETS_ZIP`.

### Packer & Installer (src/exec/packer/...)

Handles distribution units.
- `packer_impl.rs`: Zips a directory into a `.aipack` file after validating `pack.toml`.
- `installer_impl.rs`: Resolves `PackUri` (Repo, Local, Http), downloads if necessary, validates versions/prereleases, and extracts to `~/.aipack-base/pack/installed/`.
- `pack_toml.rs`: Strictly validates version format and namespace/name identity.

### Execution Command Run (src/exec/exec_cmd_run.rs)

The bridge between the CLI/Executor and the `run_agent` core.
- Handles input normalization (merging `-i` inputs and `-f` file globs).
- Manages `RunRedoCtx` for re-execution.
- Implements `exec_run_watch` using the `simple-fs` watcher.

### Self Management (src/exec/exec_cmd_xelf/...)

Handles the `aip self` command group.
- `xelf_setup`: Copies binary to `~/.aipack-base/bin` and updates shell profile (`.zshenv`, `.bashrc`, or Windows PATH).
- `xelf_update`: Checks remote `latest.toml`, downloads archive, and triggers `self setup` from the new binary.

## Common Utils (src/exec/support.rs)

- `get_available_api_keys()`: Checks environment for standard AI keys (OPENAI_API_KEY, etc.).
- `open_vscode(path)`: Legacy helper for opening files (prefer `support::editor::open_file_auto`).

## Lifecycle of a Run

1. `main.rs` parses `CliArgs`.
2. `Executor` receives `ExecActionEvent::CmdRun(args)`.
3. `exec_run` initializes the `Runtime`.
4. `do_run` expands globs into `FileInfo` objects.
5. `run_agent` is called.
6. `RunRedoCtx` is stored in the executor for potential future `Redo` actions.
