use macro_rules_attribute::derive_alias;

derive_alias! {
	// Basic compare (no float)
	#[derive(Cmp!)] = #[derive(PartialEq, Eq, PartialOrd, Ord)];

	// Basic hash (e.g., for hash keys)
	//#[derive(Hash!)] = #[derive(PartialEq, Eq, Hash)];

	// For enum as str
	#[derive(EnumAsStr!)] = #[derive(strum::IntoStaticStr, strum::AsRefStr)];
}
