use crate::hub::HubEvent;
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
	Action(ActionEvent),

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

#[derive(Debug)]
pub enum ActionEvent {
	Quit,
	Redo,
	CancelRun,
}

#[allow(unused)]
#[derive(Debug)]
pub enum DataEvent {
	DataChange,
}
