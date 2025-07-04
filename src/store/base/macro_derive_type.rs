/// Macro to add all of the derives for simple struct tuple data value.
///
/// - $vis:vis matches a visibility specifier (like pub),
/// - $name:ident matches an identifier (the struct name),
/// - $type:ty matches a type.
///
/// The macro generates the struct with the specified name and type,
/// and adds the specified attributes to it.
#[macro_export]
macro_rules! derive_simple_struct_type {
    ($(#[$meta:meta])* $vis:vis struct $name:ident($type:ty);) => {
        $(#[$meta])*
        // #[cfg_attr(feature = "for-ts", derive(schemars::JsonSchema))]
        #[derive(
            Clone,
            Copy,
            Debug,
            Hash,
            Eq,
            PartialEq,
            PartialOrd,
            derive_more::From,
            derive_more::Into,
            derive_more::Display,
            derive_more::Deref,
            // serde::Serialize,
            // serde::Deserialize,
            modql::SqliteFromValue,
            modql::SqliteToValue,
            // modql::field::SeaFieldValue,
        )]
        $vis struct $name($type);
    };
}

#[macro_export]
macro_rules! derive_simple_enum_type {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($variant:ident),* $(,)? }) => {
        $(#[$meta])*
        #[derive(
            Clone,
            Copy,
            Debug,
            Hash,
            Eq,
            PartialEq,
            PartialOrd,
            derive_more::Display,
            modql::SqliteFromValue,
            modql::SqliteToValue,
            // Add or remove derives as appropriate for your use-case
            // e.g., serde::Serialize, serde::Deserialize,
        )]
        $vis enum $name {
            $(
                $variant,
            )*
        }
    };
}
