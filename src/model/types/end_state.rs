use crate::model::ScalarEnum;
use macro_rules_attribute as mra;

#[mra::derive(Debug, ScalarEnum!)]
pub enum EndState {
	Ok,
	Err,
	Cancel,
	Skip,
}
