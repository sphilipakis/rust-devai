use crate::script::LuaValueExt;
use csv::{ReaderBuilder, Trim};
use mlua::{FromLua, Lua, Value};
use std::collections::HashMap;

/// CSV options bag used by `aip.csv` functions.
///
/// All fields are optional; when `nil` is provided for options, defaults are applied in the caller.
#[derive(Default, Clone)]
pub struct CsvOptions {
	/// Field delimiter. Default: "," (first byte used).
	pub delimiter: Option<String>,

	/// Quote character. Default: "\"" (first byte used).
	pub quote: Option<String>,

	/// Escape character. Default: "\"" (first byte used).
	pub escape: Option<String>,

	/// Whether to trim whitespace from fields. Default: false.
	pub trim_fields: Option<bool>,

	/// Whether the first row is a header. Default: false (true for `aip.file.load_csv`).
	pub has_header: Option<bool>,

	/// Map { key: label } for renaming headers/keys.
	pub header_labels: Option<HashMap<String, String>>,

	/// Whether to skip empty lines during parsing. Default: true.
	pub skip_empty_lines: Option<bool>,

	/// Comment character prefix (e.g., "#"). Default: none (if provided, only the first byte is used).
	pub comment: Option<String>,

	/// Writing only: Suppress header emission even if headers are available. Default: false.
	pub skip_header_row: Option<bool>,
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
				let skip_header_row = table.x_get_bool("skip_header_row");

				let header_labels = if let Some(val) = table.x_get_value("header_labels") {
					match val {
						Value::Table(t) => {
							let mut map = HashMap::new();
							for pair in t.pairs::<String, String>() {
								let (k, v) = pair.map_err(|e| mlua::Error::FromLuaConversionError {
									from: "Table",
									to: "HashMap<String, String>".to_string(),
									message: Some(format!("Invalid header_labels: {}", e)),
								})?;
								map.insert(k, v);
							}
							Some(map)
						}
						_ => None,
					}
				} else {
					None
				};

				Ok(CsvOptions {
					delimiter,
					quote,
					escape,
					trim_fields,
					has_header,
					header_labels,
					skip_empty_lines,
					comment,
					skip_header_row,
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
