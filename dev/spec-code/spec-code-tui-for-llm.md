# AIPack TUI Specification for LLM

This document provides a concise reference for the AIPack TUI architecture, types, and APIs to facilitate code generation and enhancement.

## Enums & Core Types

### Events (src/tui/core/event/...)

```rust
pub enum AppEvent {
    DoRedraw,
    Term(crossterm::event::Event),
    Action(ActionEvent),
    Data(DataEvent),
    Hub(HubEvent),
    Tick(i64),
}

pub enum ActionEvent {
    Quit, Redo, CancelRun,
    Scroll(ScrollDir), ScrollPage(ScrollDir), ScrollToEnd(ScrollDir),
}

pub enum ScrollDir { Up, Down }
```

### Actions (src/tui/core/types/action.rs)

```rust
pub enum Action {
    Quit, Redo, CancelRun,
    ToggleRunsNav,
    CycleTasksOverviewMode,
    GoToTask { task_id: Id },
    ToClipboardCopy(String),
    ShowText,
}
```

### Scrolling & Navigation

```rust
pub enum ScrollIden {
    RunsNav, TasksNav, TaskContent, OverviewContent,
}

pub enum RunTab { Overview, Tasks }

pub enum OverviewTasksMode { Auto, List, Grid }
```

## AppState API

`AppState` is the primary interface for views. Logic is split into multiple `impl_` files.

### Main Accessors (src/tui/core/app_state/...)
- `mm() -> &ModelManager`: Database access.
- `show_runs() -> bool`: Is the runs sidebar visible.
- `run_tab() -> RunTab`: Current active tab.
- `overview_tasks_mode() -> OverviewTasksMode`.
- `debug_clr() -> u8`: Debug color index (for live UI testing).

### Run & Task Data
- `run_items() -> &[RunItem]`: All runs in view.
- `current_run_item() -> Option<&RunItem>`.
- `tasks() -> &[Task]`: Tasks for current run.
- `current_task() -> Option<&Task>`.
- `task_idx() -> Option<usize>`.

### Model Status (impl_model_state.rs)
- `current_run_has_prompt_parts() -> Option<bool>`.
- `current_run_has_task_stages() -> Option<bool>`.
- `current_run_has_skip() -> bool`.

### Formatting & UI Getters (impl_fmt.rs)
- `current_run_duration_txt()`, `current_run_cost_fmt()`.
- `current_run_concurrency_txt()`, `current_run_agent_name()`, `current_run_model_name()`.
- `current_task_model_name()`, `current_task_cost_fmt()`, `current_task_duration_txt()`.
- `current_task_prompt_tokens_fmt()`, `current_task_completion_tokens_fmt()`.
- `current_task_cache_info_fmt() -> Option<String>`.

### Mouse API (impl_mouse.rs)
- `mouse_evt() -> Option<MouseEvt>`: Current frame event.
- `last_mouse_evt() -> Option<MouseEvt>`: Last known mouse position/state.
- `is_mouse_over(Rect) -> bool`.
- `is_mouse_up_only() -> bool`: True if mouse released with no modifiers.

### Scrolling Management (impl_scroll.rs)
- `set_scroll_area(ScrollIden, Rect)`: Register widget area for scroll detection.
- `set_scroll(ScrollIden, u16)`, `get_scroll(ScrollIden) -> u16`.
- `clamp_scroll(ScrollIden, line_count: usize) -> u16`: Clamps and returns current scroll.

### Actions & Redraw
- `set_action(Action)`: Trigger an action (processed in `state_processor.rs`).
- `trigger_redraw()`: Request frame refresh.

## Interaction & Components

### LinkZones (src/tui/core/types/link_zone.rs)
Used to handle hover/click on specific text regions.
- `push_link_zone(rel_line, start, count, Action)`: Adds clickable zone.
- `start_group() -> u32`: Starts a group for multi-line hover (e.g., whole section).
- `push_group_zone(...)`: Adds zone to a group.
- `is_mouse_over(area, scroll, mouse_evt, spans) -> Option<&mut [Span]>`.

### Shared Components (src/tui/view/comp/...)
- `el_running_ico(state)`: Status icon (▶, ✔, ✘, etc.).
- `ui_for_marker_section_str(content, (label, style), max_width, prefix)`: Standard indented section with marker.
- `ui_for_logs_with_hover(...)`, `ui_for_pins_with_hover(...)`, `ui_for_err_with_hover(...)`.

## Styling & Traits

### UiExt Trait (src/tui/support/ui_ext.rs)
- `x_bg(Color)`, `x_fg(Color)`, `x_width()`.

### RectExt Trait (src/tui/view/support/rect_ext.rs)
- `x_margin(u16)`, `x_h_margin(u16)`.
- `x_row(u16) -> Rect`: 1-indexed row selector (height 1).
- `x_top_right(w, h)`, `x_bottom_right(w, h)`.
- `x_move_down(y)`, `x_shrink_from_top(h)`.

### Common Styles (src/tui/view/style/styles_base.rs)
- `STL_FIELD_LBL`, `STL_FIELD_VAL`: For key-value headers.
- `STL_SECTION_MARKER`: Default marker style.
- `STL_SECTION_MARKER_INPUT`, `STL_SECTION_MARKER_OUTPUT`, `STL_SECTION_MARKER_AI`, `STL_SECTION_MARKER_SKIP`, `STL_SECTION_MARKER_ERR`.
- `STL_TAB_DEFAULT`, `STL_TAB_ACTIVE`, `STL_TAB_DEFAULT_HOVER`.
- `CLR_BKG_BLACK`, `CLR_TXT_WHITE`, `CLR_TXT_BLUE`, `CLR_TXT_HOVER_TO_CLIP`.
