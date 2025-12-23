# AIPack Model Specification for LLM

This document provides a concise reference for the AIPack `model` module architecture, entity relationships, and the BMC (Base Model Controller) pattern.

## Core Architecture

The `model` module manages a SQLite in-memory database using `rusqlite` and `modql`. It is organized into layers: `db` (engine), `base` (generic CRUD), `entities` (business logic), and `types` (common primitives).

### ModelManager (src/model/model_manager.rs)

- **ModelManager**: Owns the `Db` instance. Passed throughout the system via `Runtime`.
- **OnceModelManager**: A singleton provider used by the `Executor` to ensure a single shared database instance.
- **trim()**: Clears main tables (`run`, `task`, `log`). Typically called at the start of new top-level runs.

### Database Schema (src/model/db/rt_db_setup.rs)

- **Main Tables**: `run`, `task`, `log`, `err`, `prompt`, `pin`.
- **Content Tables**: `inout` (stores large task inputs/outputs, linked by `task_uid`).
- **Keys**: All tables use `id` (INTEGER PRIMARY KEY) for internal joins and `uid` (BLOB/UUID) for external/Lua references.

## The BMC Pattern (src/model/base/...)

Every entity (e.g., `Run`, `Task`) has a corresponding BMC struct (e.g., `RunBmc`) that implements `DbBmc`.

### Generic CRUD (crud_fns.rs)
- `create<MC>(mm, fields)`: Adds `uid`, `ctime`, `mtime`.
- `update<MC>(id, fields)`: Updates `mtime`.
- `get<MC, E>(id)`: Fetches a single entity by ID.
- `list<MC, E>(list_options, filter)`: Fetches a collection with ordering and filtering.
- `batch_create<MC>(mm, items)`: High-performance insertion within a single transaction.

## Entity Modules (src/model/entities/...)

### Run (run.rs)
Tracks the execution of an Agent. Stores timestamps for phases (`ba` - Before All, `tasks`, `aa` - After All), agent metadata, and aggregate costs.

### Task (task.rs)
Tracks individual inputs/files within a run. Stores detailed AI usage tokens (`tk_prompt_total`, `tk_completion_total`), pricing, and model overrides.
- `update_input` / `update_output`: Logic for storing short summaries in the `task` table while moving large content to the `inout` table.

### Log (log.rs)
Captures execution events. `LogKind` includes `RunStep`, `SysInfo`, `AgentPrint`, `AgentSkip`, etc. Linked to `run_id` and optionally `task_id`.

### Pin (pin.rs)
Persistent UI markers or summary info created by agents via `aip.task.pin`. Uses `save_run_pin` (upsert logic by `iden`) to avoid duplicates.

## Common Types (src/model/types/...)

- **Id / EpochUs**: `ScalarStruct` wrappers for `i64`.
- **Stage**: `BeforeAll`, `Data`, `Ai`, `AiGen`, `Output`, `AfterAll`.
- **RunStep**: Granular markers like `BaStart`, `TaskAiGenStart`, `AaEnd`.
- **RunningState**: `Waiting`, `Running`, `Ended(Option<EndState>)`.
- **EndState**: `Ok`, `Err`, `Cancel`, `Skip`.
- **TypedContent**: Handles `Json` or `Text` content, managing UUID generation and auto-truncation for "short" previews.
