use crate::store::Id;

#[derive(Debug, Clone, Copy)]
pub enum Action {
	// Go to the tasks tab and select this task_id
	GoToTask { task_id: Id },
}
