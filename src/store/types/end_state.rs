use crate::derive_simple_enum_type;

derive_simple_enum_type! {
pub enum EndState {
	Ok,
	Err,
	Cancel,
	Skip
}
}
