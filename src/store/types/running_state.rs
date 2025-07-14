use crate::store::EndState;

#[derive(Debug)]
pub enum RunningState {
	Waiting,
	Running,
	Ended(Option<EndState>), // see EndState
}
