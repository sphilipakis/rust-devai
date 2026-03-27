# AIPack Flow Specification

This document describes the `aip.flow` module and how it controls the execution lifecycle of an Agent.

## Overview

The `aip.flow` module provides functions that return "Directive Tables". These tables are recognized by the Agent Executor to alter the normal sequential flow of stages.

When a directive is returned:
1. It immediately interrupts the current stage.
2. It propagates back to the Agent Executor.
3. The Executor performs the requested action (Skip, Redo, etc.).

## Directives

### `aip.flow.skip(reason?)`

Instructs the executor to stop processing the current task and move to the next one.
- **BeforeAll**: Skips the entire run.
- **Data**: Skips the current input item.
- **Output**: Skips finishing the current task (output is recorded as a skip).

### `aip.flow.redo_run()`

Instructs the executor to finish the current active tasks and then restart the entire agent execution from the beginning.
- This is useful for self-correcting agents or when the agent needs to reload its own code/configuration.
- This directive is valid only when returned from:
  - `# Before All`
  - `# After All`
- Returning it from:
  - `# Data`
  - `# Output`
  fails with an explicit stage error:
  - `aip.flow.redo_run() can be returned only from # Before All or # After All stages.`
- Redo is propagated as a run-level request, then scheduled by the executor as a follow-up `Redo` action.
- Automatic redo chaining is supported, meaning a redo-triggered rerun can request redo again and continue the cycle.
- Before dispatching the next redo action, the executor waits `500ms`.
- The initial top-level run starts with redo count `0`.
- Each accepted redo transition increments the redo count by `1` for the next rerun.
- The current redo count is exposed to Lua through `CTX.REDO_COUNT`.
- `CTX.REDO_COUNT` is absent for a normal first run and present for redo-chain reruns.

### `aip.flow.before_all_response(data)`

Used in `before_all` to override inputs or options for the run.

### `aip.flow.data_response(data)`
Used in `data` to override the input or options for a single task.

## Flow Interruption Logic

### Stage-based behavior

- **BeforeAll**: If `skip` or `redo` is returned, the `Data` and `Ai` stages are never entered.
- **Data**: If `skip` is returned for a specific input, the `Ai` and `Output` stages for that input are bypassed. If `redo` is returned, the stage fails with the explicit error `aip.flow.redo_run() can be returned only from # Before All or # After All stages.` because redo is not allowed from `# Data`.
- **Output**: If `redo` is returned, the stage fails with the explicit error `aip.flow.redo_run() can be returned only from # Before All or # After All stages.` because redo is not allowed from `# Output`.
- **AfterAll**: If `redo` is returned, the overall run is marked for redo.

### Redo lifecycle

The redo behavior is split between the run pipeline and the executor.

- The run pipeline detects `_aipack_ = { kind = "Redo" }` and surfaces this as `redo_requested = true` in the run response.
- The executor owns the redo scheduling policy.
- The run context carries the current redo count for the active run.
- When a run or redo-run completes successfully with `redo_requested = true`, the executor:
  - refreshes and stores the latest `RunRedoCtx`
  - increments the redo count for the next rerun
  - waits `500ms`
  - enqueues the next `ExecActionEvent::Redo`
- This applies to:
  - the initial `Run` path
  - the `Redo` path itself

This design ensures redo is not a one-shot follow-up. As long as each completed rerun keeps requesting redo, the executor continues to schedule the next redo cycle.

### Redo count flow

- A normal first run begins a new redo chain with count `0`.
- If `aip.flow.redo_run()` is returned from an allowed stage, the current run completes and requests a redo.
- The next rerun receives redo count `1`.
- If that rerun requests redo again, the following rerun receives redo count `2`, and so on.
- Starting a new top-level run resets the redo count to `0`.
- Invalid redo attempts from unsupported stages do not advance the count because no redo transition is accepted.

### Design considerations

- Redo scheduling is intentionally centralized in the executor instead of being recursively handled inside the run pipeline.
- The stored redo context must be refreshed after each successful rerun so the next redo uses the latest execution context.
- If a redo execution fails, the previous redo context should remain available for manual retry behavior.
- The `500ms` wait is a pacing delay before dispatching the next full redo event.
- Redo count is carried by run context so flow logic can observe the current chain position without coupling to executor internals.
- Resetting the count on a new top-level run prevents leakage across unrelated executions.

### Propagation
Directives are wrapped in an `_aipack_` internal table structure:

```json
{
  "_aipack_": {
    "kind": "Redo"
  }
}
```

The runtime detects these patterns in the values returned by Lua scripts and executes the corresponding logic.

