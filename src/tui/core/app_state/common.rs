use crate::model::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStage {
	Normal,
	Installing,
	Installed,
	PromptInstall(Id),
}
