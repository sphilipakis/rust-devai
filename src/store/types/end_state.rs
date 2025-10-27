use crate::store::ScalarEnumType;
use macro_rules_attribute as mra;

#[mra::derive(Debug, ScalarEnumType!)]
pub enum EndState {
	Ok,
	Err,
	Cancel,
	Skip,
}
