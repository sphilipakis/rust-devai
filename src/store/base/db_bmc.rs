pub trait DbBmc {
	const TABLE: &'static str;

	fn table_ref() -> &'static str {
		Self::TABLE
	}
}
