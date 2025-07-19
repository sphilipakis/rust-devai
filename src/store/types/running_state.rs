use crate::store::EndState;

#[derive(Debug, Clone)]
pub enum RunningState {
	NotScheduled,
	Waiting,
	Running,
	Ended(Option<EndState>), // see EndState
}
