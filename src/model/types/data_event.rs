use crate::model::Id;

// region:    --- Types

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
	#[allow(unused)] // for now
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

// endregion: --- Types
