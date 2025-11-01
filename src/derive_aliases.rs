use macro_rules_attribute::derive_alias;

derive_alias! {
	// When need Ord (if need Hash as well, add it to the derive)
	#[derive(Cmp!)] = #[derive(PartialEq, Eq, PartialOrd, Ord)];

	// When just need to have hash only (no ex)
	//#[derive(Hash!)] = #[derive(PartialEq, Eq, Hash)];

	// For enum as str
	#[derive(EnumAsStr!)] = #[derive(strum::IntoStaticStr, strum::AsRefStr)];

}
