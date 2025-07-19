use crate::store::EndState;

#[derive(Debug, Clone)]
pub enum RunningState {
	NotScheduled,
	Waiting,
	Running,
	Ended(Option<EndState>), // see EndState
}

// region:    --- Froms

impl From<EndState> for RunningState {
	fn from(value: EndState) -> Self {
		match value {
			EndState::Ok => RunningState::Ended(Some(EndState::Ok)),
			EndState::Cancel => RunningState::Ended(Some(EndState::Cancel)),
			EndState::Skip => RunningState::Ended(Some(EndState::Skip)),
			EndState::Err => RunningState::Ended(Some(EndState::Err)),
		}
	}
}

// endregion: --- Froms
