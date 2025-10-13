use crate::store::Id;

#[derive(Debug, Clone)]
pub enum Action {
	// -- Global Actions
	Quit,
	Redo,
	CancelRun,
	ToggleRunsNav,
	CycleTasksOverviewMode,

	// Go to the tasks tab and select this task_id
	GoToTask {
		task_id: Id,
	},

	#[allow(unused)]
	ShowText,

	// Copy the provided text into the clipboard
	ToClipboardCopy(String),
}
