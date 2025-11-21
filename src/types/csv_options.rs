use crate::script::LuaValueExt;
use csv::{ReaderBuilder, Trim};
use mlua::{FromLua, Lua, Value};

/// CSV options bag used by `aip.csv` functions.
///
/// All fields are optional; when `nil` is provided for options, defaults are applied in the caller:
/// - delimiter: "," (first byte used)
/// - quote: "\"" (first byte used)
/// - escape: "\"" (first byte used)
/// - trim_fields: false
/// - has_header: false
/// - skip_empty_lines: true
/// - comment: none (if provided, only the first byte is used)
#[derive(Default, Clone)]
pub struct CsvOptions {
	pub delimiter: Option<String>,
	pub quote: Option<String>,
	pub escape: Option<String>,
	pub trim_fields: Option<bool>,
	pub has_header: Option<bool>,
	pub skip_empty_lines: Option<bool>,
	pub comment: Option<String>,
}

impl FromLua for CsvOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(CsvOptions::default()),
			Value::Table(table) => {
				let delimiter = table.x_get_string("delimiter");
				let quote = table.x_get_string("quote");
				let escape = table.x_get_string("escape");
				let trim_fields = table.x_get_bool("trim_fields");
				let has_header = table.x_get_bool("has_header");
				let skip_empty_lines = table.x_get_bool("skip_empty_lines");
				let comment = table.x_get_string("comment");

				Ok(CsvOptions {
					delimiter,
					quote,
					escape,
					trim_fields,
					has_header,
					skip_empty_lines,
					comment,
				})
			}
			other => Err(mlua::Error::FromLuaConversionError {
				from: other.type_name(),
				to: "CsvOptions".to_string(),
				message: Some("Expected nil or a table for CsvOptions".into()),
			}),
		}
	}
}

impl CsvOptions {
	/// Build a `csv::ReaderBuilder` applying only the values explicitly set in `CsvOptions`.
	/// Defaults are left to `ReaderBuilder` when the corresponding option is `None`.
	pub fn into_reader_builder(self) -> ReaderBuilder {
		let mut builder = ReaderBuilder::new();

		if let Some(b) = self.delimiter.as_ref().and_then(|s| s.as_bytes().first().copied()) {
			builder.delimiter(b);
		}

		if let Some(b) = self.quote.as_ref().and_then(|s| s.as_bytes().first().copied()) {
			builder.quote(b);
		}

		if let Some(b) = self.escape.as_ref().and_then(|s| s.as_bytes().first().copied()) {
			builder.escape(Some(b));
		}

		if let Some(trim_fields) = self.trim_fields {
			match trim_fields {
				true => {
					builder.trim(Trim::All);
				}
				false => {
					builder.trim(Trim::None);
				}
			}
		}

		if let Some(has_header) = self.has_header {
			builder.has_headers(has_header);
		}

		if let Some(b) = self.comment.as_ref().and_then(|s| s.as_bytes().first().copied()) {
			builder.comment(Some(b));
		}

		builder
	}

	/// Build a `csv::WriterBuilder` applying only the values explicitly set in `CsvOptions`.
	/// Defaults are left to `WriterBuilder` when the corresponding option is `None`.
	pub fn into_writer_builder(self) -> csv::WriterBuilder {
		let mut builder = csv::WriterBuilder::new();

		if let Some(b) = self.delimiter.as_ref().and_then(|s| s.as_bytes().first().copied()) {
			builder.delimiter(b);
		}

		if let Some(b) = self.quote.as_ref().and_then(|s| s.as_bytes().first().copied()) {
			builder.quote(b);
		}

		if let Some(b) = self.escape.as_ref().and_then(|s| s.as_bytes().first().copied()) {
			builder.escape(b);
		}

		builder
	}
}
