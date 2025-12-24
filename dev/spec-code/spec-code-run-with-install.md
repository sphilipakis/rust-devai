# AIPack Run-with-Install Specification for LLM

This document provides a concise reference for the AIPack "Run-with-Install" architecture, which handles the detection of missing agent packs during a `run` command and orchestrates an interactive user prompt and installation flow.

## Overview

When a user executes `aip run namespace@pack/agent`, if the pack is not found locally, the system transitions into an interactive installation workflow before proceeding with the run.

## Architectural Flow

### 1. Detection (src/exec/executor.rs)
The `Executor` receives `ExecActionEvent::CmdRun(run_args)`.
1. `find_agent` fails.
2. The system checks `looks_like_pack_ref(&agent_name)`.
3. If it looks like a pack, the `Executor` creates a `WorkKind::Install` database record.
4. The record stores metadata in `WorkInstallData`:
    - `pack_ref`: The base `namespace@pack` identity.
    - `run_args`: The original `RunArgs` (serialized).
    - `needs_user_confirm`: Set to `true`.
5. The `Executor` publishes `HubEvent::RtModelChange` and terminates the current action.

### 2. TUI Coordination (src/tui/core/app_state/state_processor.rs)
The TUI `state_processor` monitors the `work` table during every tick.
1. It detects an active `WorkKind::Install` where `needs_user_confirm` is `true`.
2. It transitions the application to `AppStage::PromptInstall(work_id)`.
3. `src/tui/view/install_view.rs` renders a centered dialog overlay with "Install" and "Cancel" buttons.

### 3. User Interaction
- **Confirmation**: Sets `UiAction::WorkConfirm(id)` -> `AppActionEvent::WorkConfirm(id)` -> `ExecActionEvent::WorkConfirm(id)`.
- **Cancellation**: Sets `UiAction::WorkCancel(id)` -> `AppActionEvent::WorkCancel(id)` -> `ExecActionEvent::WorkCancel(id)`.

### 4. Installation & Resumption (src/exec/executor.rs)
Upon receiving `WorkConfirm(id)`:
1. The `Executor` updates the `Work` record: `needs_user_confirm = false` and `start = now()`.
2. The TUI automatically transitions to `AppStage::Installing` (showing progress).
3. The `Executor` calls `exec_install` (via `src/exec/exec_cmd_install.rs`).
4. **On Success**:
    - Updates `Work` record to `EndState::Ok`.
    - Extracts the original `RunArgs` from the metadata.
    - Sends a new `ExecActionEvent::CmdRun(original_args)` to the queue.
5. The TUI transitions to `AppStage::Installed` (success checkmark) then back to `Normal` after a timeout.

## Data Contracts

### WorkInstallData (src/exec/cli/args.rs)
The shared structure for the `work.data` JSON column.
```rust
pub struct WorkInstallData {
    pub pack_ref: String,
    pub run_args: Option<Value>, // Serialized RunArgs
    pub needs_user_confirm: bool,
}
```

## Key Files & Responsibilities

- `src/exec/executor.rs`: Orchestrates the lifecycle (detection, installation, and resumption).
- `src/exec/cli/args.rs`: Defines the `RunArgs` and `WorkInstallData` serialization formats.
- `src/tui/core/app_state/state_processor.rs`: Bridges the database state to the TUI `AppStage`.
- `src/tui/core/app_state/common.rs`: Defines the `AppStage::PromptInstall` variant.
- `src/tui/view/install_view.rs`: Renders the interactive prompt and installation status dialogs.
- `src/model/entities/work.rs`: BMC and entity for tracking background operations.
- `src/types/pack_ref.rs`: Provides `looks_like_pack_ref` and `PackRef` parsing.

## User Flow Summary
1. `aip run demo@proof`
2. Dialog appears: "Agent pack 'demo@proof' is not installed. Do you want to install it now? [Install] [Cancel]"
3. User presses `i` or clicks "Install".
4. Dialog updates: "Installing pack ... demo@proof".
5. Dialog updates: "✔ Installed".
6. Success overlay disappears, and the `demo@proof` agent execution begins automatically.
