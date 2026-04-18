use crate::exec::cli::RunArgs;
use crate::hub::HubEvent;
use crate::model::Id;
use derive_more::From;

/// The main application event enum.
///
/// This enum encapsulates all possible events that can occur in the application,
/// serving as a central point for event handling. It includes terminal UI events
/// and custom application-specific data events.
///
/// The `#[derive(From)]` allows for convenient conversion from its variant types
/// into `AppEvent`.
#[derive(From, Debug)]
pub enum AppEvent {
	// When a UI Component changed a state that might affect other previously rendered states
	DoRedraw,

	// Terminal Read Events
	#[from]
	Term(crossterm::event::Event),

	// App Action Event
	#[from]
	Action(AppActionEvent),

	// Data Event
	#[from]
	Data(DataEvent),

	// Hub Event
	#[from]
	Hub(HubEvent),

	// Just a tick event (with a now time micro of when this was sent)
	Tick(i64),
}

impl AppEvent {
	pub fn is_refresh_event(&self) -> bool {
		matches!(self, AppEvent::DoRedraw | AppEvent::Data(_) | AppEvent::Hub(_))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
	Run,
	Task,
	Log,
	Err,
	Prompt,
	Pin,
	Ucontent,
	Work,
	Inout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAction {
	Created,
	Updated,
	Deleted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelIds {
	pub run_id: Option<Id>,
	pub task_id: Option<Id>,
	pub log_id: Option<Id>,
	pub err_id: Option<Id>,
	pub prompt_id: Option<Id>,
	pub pin_id: Option<Id>,
	pub ucontent_id: Option<Id>,
	pub work_id: Option<Id>,
	pub inout_id: Option<Id>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataEvent {
	pub entity: EntityType,
	pub action: EntityAction,
	pub id: Option<Id>,
	pub rel_ids: RelIds,
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollDir {
	Up,
	Down,
}

/// Represents a **System Command** sent via transport (`AppTx`).
/// It is event-driven and represents a discrete instruction for the main loop.
///
/// See `dev/spec-code/spec-code-tui.md` for the architectural rationale and flow.
#[derive(Debug)]
pub enum AppActionEvent {
	Quit,
	Redo,
	CancelRun,
	Scroll(ScrollDir),
	ScrollPage(ScrollDir),
	ScrollToEnd(ScrollDir),
	WorkConfirm(Id),
	WorkCancel(Id),
	Run(RunArgs),
}


