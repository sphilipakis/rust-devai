use crate::dir_context::PackDir;
use derive_more::From;

#[derive(Debug, From)]
pub enum PrintEvent {
	#[from]
	PackList(Vec<PackDir>),
}
