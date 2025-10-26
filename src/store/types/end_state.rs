use crate::derive_simple_enum_type;
use macro_rules_attribute::apply;

#[apply(derive_simple_enum_type)]
pub enum EndState {
	Ok,
	Err,
	Cancel,
	Skip,
}
