use crate::Result;
use std::borrow::Cow;
use std::io::BufRead;
use std::path::Path;
use std::{fs, io, str};

/// Enum to represent Cow Lines iterator over &str or File Buffer (for now)
/// Note: This allows to have static dispatch
pub enum CowLines<'a> {
	StrLines(str::Lines<'a>),
	FileLines(io::Lines<io::BufReader<fs::File>>),
}

impl<'a> Iterator for CowLines<'a> {
	type Item = Cow<'a, str>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			CowLines::StrLines(lines) => lines.next().map(Cow::Borrowed),
			CowLines::FileLines(lines) => lines.next().map(|line| Cow::Owned(line.unwrap())),
		}
	}
}

/// Constructors
impl<'a> CowLines<'a> {
	pub fn from_str(content: &'a str) -> Self {
		CowLines::StrLines(content.lines())
	}

	pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
		let file = std::fs::File::open(path)?;
		let reader = io::BufReader::new(file);
		Ok(CowLines::FileLines(reader.lines()))
	}
}

/// Utilities
impl<'a> CowLines<'a> {
	/// Joins a `Vec<Cow<str>>` into a single `String` efficiently.
	pub fn join(&mut self, separator: &str) -> String {
		let parts: Vec<Cow<str>> = self.collect();
		parts.join(separator)
	}
}