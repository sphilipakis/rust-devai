# Dev Chat

Add a new `## Request: _user_ask_title_concise_` with the answer below (concise title). Use markdown sub-headings for sub sections. Keep this top instruction in this file. 

## Request: Support for Hidden Sub-Agent Runs

This feature allows sub-agents to execute without cluttering the TUI Run Navigation panel unless they encounter an error.

### Design Summary

- **Lua API**: Add a `hidden` boolean property to the `RunAgentOptions` table in `aip.agent.run(name, { hidden = true, ... })`.
- **Persistence**: Update the `run` table in the SQLite schema to include a `hidden` column (INTEGER).
- **Model**: Update the `Run` entity and its `Create`/`Update` variants to include the `hidden` field.
- **TUI Filtering**:
  - A run is "effectively hidden" if it is marked `hidden` and has not failed (`end_state != EndState::Err`).
  - If a parent is "effectively hidden", all its children are also hidden by default.
  - However, if any child (direct or indirect) is visible (e.g., due to an error), all its ancestors must be shown to maintain tree integrity.
  - A "Show Hidden Runs" toggle (key `s`) will override this logic and show all runs.
- **Visibility during execution**: Hidden runs remain hidden while running (unless they fail).

### Implementation Plan

#### 1. Configuration & Options
Update `src/types/run_agent_options.rs` to include `hidden: Option<bool>` in `RunAgentOptions`.
Update `src/script/aip_modules/aip_agent.rs` to extract this field from the Lua table.

#### 2. Database Schema
Update `src/model/db/rt_db_setup.rs` to add the `hidden` column to the `run` table definition.

#### 3. Domain Model
Update `src/model/entities/run.rs` to add `hidden: Option<bool>` to `Run`, `RunForCreate`, and `RunForUpdate`.

#### 4. TUI Logic (Store & Filter)
Update `src/tui/core/app_state/` to add `show_hidden_runs: bool`.
Update `RunItemStore` or `AppState` logic to compute the visible subset of `run_items` based on the `hidden` property, `end_state`, hierarchy rules, and the `show_hidden_runs` toggle.

#### 5. TUI UI (Toggle)
In `src/tui/view/runs_nav_view.rs`, add a bottom bar hint `[s] show hidden` (only when at least one hidden run exists). Handle the `s` key to toggle `AppState::show_hidden_runs`.

## Request: Review Request Completeness for Hidden Sub-Agent Runs

### Overall assessment

Yes, the request is mostly complete and clear, the implementation intent is understandable end-to-end.

### What is already clear

- The product behavior is clearly defined:
  - hidden sub-runs should not appear in nav by default
  - failed hidden runs must become visible
  - parent-child visibility propagation rules are specified
  - manual override via toggle is defined
- Storage and API touchpoints are identified:
  - Lua option (`hidden`)
  - DB column (`run.hidden`)
  - model fields (`Run`, `RunForCreate`, `RunForUpdate`)
- UX direction is clear:
  - toggle key `s`
  - hint shown conditionally

### Open points to clarify before coding

- `aip.agent.run` scope:
  - Should `hidden` be honored only for sub-agents, or also when calling top-level agent runs from Lua contexts where that could happen?
  - ANSWER: By design, a aip.agent.run is alwsy a nested agent, so, this will be the only way i can be stored. So we do not do anything special.
- Defaults:
  - Confirm default is visible (`hidden = false`/`None`) for all runs unless explicitly set.
  - ANSWER: Yes, hiden is None, which is false. 
- Existing DB migration strategy:
  - For already existing DBs, should we run an `ALTER TABLE run ADD COLUMN hidden INTEGER` path, or is recreate-only acceptable in this phase?
  - ANSWER: NO, we do not need db migration, it's in memory db, so jus update the schema sql following the order/pattern
- Parent visibility rule wording:
  - Confirm this exact rule: if any descendant is visible, all ancestors up to root are visible, even if those ancestors are themselves hidden and non-error.
  - ANWSWER: NO, if an nested agent is hidden, then, the UI will not show this agent and the shildren in the nav run. (when error any, then, all show)
- Toggle semantics:
  - Confirm toggle label/meaning:
    - when off: filtered mode active
    - when on: show all runs including hidden non-error runs
    - ANSWER: Correct, when on all hidden show, as if the flag is not shown. 
  - confirm whether hint text should switch between `[s] show hidden` and `[s] hide hidden`.
  - ANSWER: She those are good text
- Error-only reveal timing:
  - Confirm reveal condition is strictly `end_state == Err`, not any non-`Ok` state (Cancel, Skip).
  - ANSWER: Whenver they have an error that get display as error. 
- Running hidden runs:
  - Confirm that a hidden run remains invisible while running unless it eventually errors, as already stated.
  - ANSWER: Correct.

ALSO, one thing important, is that we should have in RunFacade or somwhere, or app state, a should_show_run or something like that so that the TUI Views can use that reather than inline it in their view. 

### Minor consistency note

- In `RunForUpdate` there is currently a field `has_prompt` while table/entity naming uses `has_prompt_parts`.
  - This is unrelated to hidden runs, but worth confirming no existing naming mismatch should be addressed in same change set or left untouched.
  - ANSWER: Do not change that for now. 

### Conclusion

- You answered most of the essential product and technical questions.
- A short confirmation on the open points above will make implementation fully unambiguous.

## Request: Hidden Sub-Agent Runs, Clarity Check

### Assessment

Yes, this is clear enough to implement.

### What is now unambiguous

- `hidden` is added to `RunAgentOptions` and passed from Lua via `aip.agent.run(..., { hidden = true })`.
- Default is visible, stored as `None` in Rust model, equivalent to false.
- DB migration path is not needed, only schema update for in-memory setup.
- Hidden run visibility rule:
  - hidden subtree is not shown by default
  - if an error is present in that hidden subtree, then it becomes visible in nav
- Toggle behavior is clear:
  - off = filtered mode
  - on = show all runs
  - label can switch between `[s] show hidden` and `[s] hide hidden`
- Error reveal is error-only, not cancel/skip.
- Running hidden runs stay hidden unless error condition makes them visible.
- Visibility logic should be centralized in app state or facade (`should_show_run` style), not duplicated in views.

### Remaining blocker check

No blocker remains from product behavior perspective. Implementation can proceed.

### Suggested implementation note

- Keep the filtering API owned by app state or a dedicated facade layer, so all TUI views consume one source of truth for run visibility.

