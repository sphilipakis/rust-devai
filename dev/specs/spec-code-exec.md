# AIPack Executor Specification for LLM

This document provides a concise reference for the AIPack `exec` module architecture, command events, and lifecycle management.

## Core Architecture

The `exec` module follows a decoupled command pattern. The `Executor` runs an asynchronous loop that processes `ExecActionEvent` variants.

### Executor & ExecutorTx (src/exec/executor.rs)

- **Executor**: Owns the receiver and manages the active task count. It spawns a new Tokio task for every action received.
- **ExecutorTx**: A cloneable sender wrapper.
    - `send(ExecActionEvent)`: Async send (preferred).
    - `send_sync(ExecActionEvent)`: Synchronous send (flume).
    - `send_sync_spawn_and_block(ExecActionEvent)`: Used when calling from a sync context where the event must be processed in parallel (e.g., Lua module calls like `aip.agent.run`).

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
    CmdNew(NewArgs),

    // -- Interactive/UI Commands
    OpenAgent, // Opens current agent in editor
    Redo,      // Re-executes the last run
    CancelRun, // Signals cancellation via CancelTrx

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

## Lifecycle of a Run

1. `main.rs` parses `CliArgs`.
2. `Executor` receives `ExecActionEvent::CmdRun(args)`.
3. **Auto-Install Detection**: If `find_agent` fails and the name contains `@`, the executor creates a `WorkKind::Install` record and calls `exec_install`.
4. `exec_run` initializes the `Runtime`.
5. `do_run` expands globs into `FileInfo` objects.
6. `run_agent` is called.
7. `RunRedoCtx` is stored in the executor for potential future `Redo` actions.

## Request: Atomic DB Transactions with Locked Closures

### Problem Analysis

The current implementation of transactions (e.g., in `batch_create` in `src/model/base/crud_fns.rs`) is not thread-safe. While the `Db` struct uses a `Mutex<Connection>`, the lock is acquired and released for every individual `db.exec(...)` call. 

In `batch_create`, the code does:
1. `db.exec("BEGIN", ...)` -> acquires lock, executes, releases lock.
2. Multiple `db.exec_returning_num(...)` -> each acquires/releases lock.
3. `db.exec("COMMIT", ...)` -> acquires lock, executes, releases lock.

Between any of these steps, another thread could acquire the lock and execute a statement, potentially interfering with the transaction state.

### Proposed Implementation Plan

1.  **Preserve existing `Db` API**: Keep `pub fn exec(...)`, `pub fn fetch_all(...)`, etc., as they are. They will continue to acquire a lock for single-shot operations.
2.  **Internal Refactoring**: Move the core logic of these methods into internal functions or a trait that operates directly on a `&Connection`.
3.  **Add `pub fn exec_tx`**: This method will manage the transaction lifecycle and the mutex lock:
    - It acquires the `Mutex` lock once.
    - It executes `BEGIN`.
    - It provides a closure (`work_fn`) with a "Transaction Context" object (e.g., `TxDb`) that exposes the same high-level API (`exec`, `fetch_all`, etc.) but operates on the already-locked connection without re-acquiring the mutex (preventing deadlocks).
    - It handles `COMMIT` on success and `ROLLBACK` on error.
4.  **Update `batch_create`**: Refactor `src/model/base/crud_fns.rs` to use the new `db.exec_tx` method.

### Example API Concept

```rust
pub fn exec_tx<F, R>(&self, work_fn: F) -> Result<R>
where F: FnOnce(&TxDb) -> Result<R> {
    let mut conn_g = self.con.lock()?; // Mutex held for the whole block
    
    // 1. BEGIN
    conn_g.execute("BEGIN", [])?;
    
    // 2. Wrap the locked connection
    let tx_db = TxDb::new(&conn_g);
    
    // 3. Exec work
    let res = work_fn(&tx_db);
    
    // 4. COMMIT or ROLLBACK
    match res {
        Ok(_) => { conn_g.execute("COMMIT", [])?; }
        Err(_) => { let _ = conn_g.execute("ROLLBACK", []); }
    }
    
    res
}
```

### Questions & Considerations

- **SQLite Savepoints**: If we need nested transactions in the future, we should use `SAVEPOINT` instead of `BEGIN/COMMIT`. For the current scope, a single top-level transaction is sufficient.
- **API Consistency**: The `TxDb` should mirror the `Db` interface to make refactoring `batch_create` and other complex operations straightforward.
