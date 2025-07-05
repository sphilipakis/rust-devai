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
}

#[derive(Debug)]
pub enum ActionEvent {
	Redo,
}

#[allow(unused)]
#[derive(Debug)]
pub enum DataEvent {
	DataChange,
}
