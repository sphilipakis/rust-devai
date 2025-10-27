use macro_rules_attribute::derive_alias;

derive_alias! {
	// Scalar Struct type for DB Primitive type wrapper
	#[derive(ScalarStructType!)] = #[derive(
		crate::Ord!,
		Clone,
		Copy,
		Hash,
		derive_more::From,
		derive_more::Into,
		derive_more::Display,
		derive_more::Deref,
		modql::SqliteFromValue,
		modql::SqliteToValue,
	)];

	#[derive(ScalarEnumType!)] = #[derive(
		crate::Ord!,
		Clone,
		Copy,
		Hash,
		derive_more::Display,
		crate::EnumAsStr!,
		modql::SqliteFromValue,
		modql::SqliteToValue,
	)];
}
