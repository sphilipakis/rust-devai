use crate::store::EndState;

#[derive(Debug)]
pub enum RunningState {
	NotScheduled,
	Waiting,
	Running,
	Ended(Option<EndState>), // see EndState
}
