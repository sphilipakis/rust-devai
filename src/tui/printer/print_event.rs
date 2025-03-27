use crate::dir_context::PackDir;
use derive_more::From;
use std::collections::HashSet;

#[derive(Debug, From)]
pub enum PrintEvent {
	#[from]
	PackList(Vec<PackDir>),

	ApiKeysStatus {
		all_keys: &'static [&'static str],
		available_keys: HashSet<String>,
	},
}

