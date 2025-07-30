use crate::types::uc;
use derive_more::From;

#[allow(unused)]
#[derive(Debug, From)]
pub enum UComp {
	Marker(uc::Marker),
}
