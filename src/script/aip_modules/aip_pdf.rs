//! Defines the `pdf` module, used in the Lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.pdf` module exposes functions to work with PDF files.
//!
//! ### Functions
//!
//! - `aip.pdf.page_count(path: string): number`
//!   Returns the number of pages in a PDF file.
//! - `aip.pdf.split_pages(path: string, dest_dir?: string): string[]`
//!   Splits a PDF into individual page files.

use crate::runtime::Runtime;
use crate::support::pdf;
use crate::types::FileInfo;
use crate::{Error, Result};
use mlua::{IntoLua, Lua, Table, Value};
use simple_fs::SPath;

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let page_count_fn = lua.create_function(move |_lua, path: String| page_count(path))?;

	let rt = runtime.clone();
	let page_split_fn = lua
		.create_function(move |lua, (path, dest_dir): (String, Option<String>)| page_split(lua, &rt, path, dest_dir))?;

	table.set("page_count", page_count_fn)?;
	table.set("split_pages", page_split_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Returns the number of pages in a PDF file.
///
/// ```lua
/// -- API Signature
/// aip.pdf.page_count(path: string): number
/// ```
///
/// ### Arguments
///
/// - `path: string` - The path to the PDF file.
///
/// ### Returns
///
/// - `number` - The number of pages in the PDF.
///
/// ### Example
///
/// ```lua
/// local count = aip.pdf.page_count("documents/report.pdf")
/// print("Page count:", count)
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The file does not exist or cannot be read.
/// - The file is not a valid PDF.
fn page_count(path: String) -> mlua::Result<usize> {
	let spath =
		SPath::from_std_path(&path).map_err(|err| Error::custom(format!("aip.pdf.page_count failed. {err}")))?;

	let doc = pdf::load_pdf_doc(&spath).map_err(|err| Error::custom(format!("aip.pdf.page_count failed. {err}")))?;

	Ok(pdf::page_count(&doc))
}

/// ## Lua Documentation
///
/// Splits a PDF into individual page files.
///
/// ```lua
/// -- API Signature
/// aip.pdf.split_pages(path: string, dest_dir?: string): list<FileInfo>
/// ```
///
/// Splits the PDF at `path` into individual single-page PDF files.
///
/// If `dest_dir` is not provided, the destination directory defaults to a folder
/// in the same location as the source PDF, named after the PDF's stem (filename without extension).
///
/// For example, if `path` is `"docs/report.pdf"`, the default destination would be `"docs/report/"`.
///
/// Each page file is named `{stem}-page-{NNNN}.pdf` where `{stem}` is the original filename
/// without extension and `{NNNN}` is a zero-padded 4-digit page number.
///
/// ### Arguments
///
/// - `path: string` - The path to the PDF file to split.
/// - `dest_dir?: string` (optional) - The destination directory for the split page files.
///   If not provided, defaults to a folder named after the PDF stem in the same directory.
///
/// ### Returns
///
/// - `list<FileInfo>` - A list of [`FileInfo`] objects for each created page file.
///
/// ### Example
///
/// ```lua
/// -- Split with default destination (creates "docs/report/" folder)
/// local pages = aip.pdf.split_pages("docs/report.pdf")
/// for _, page in ipairs(pages) do
///   print(page.path) -- e.g., "docs/report/report-page-0001.pdf"
///   print(page.name) -- e.g., "report-page-0001.pdf"
/// end
///
/// -- Split to a specific destination
/// local pages = aip.pdf.split_pages("docs/report.pdf", "output/pages")
/// for _, page in ipairs(pages) do
///   print(page.path, page.size)
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The source file does not exist or cannot be read.
/// - The file is not a valid PDF.
/// - The destination directory cannot be created.
/// - Any page cannot be saved.
fn page_split(lua: &Lua, runtime: &Runtime, path: String, dest_dir: Option<String>) -> mlua::Result<Value> {
	let pdf_path =
		SPath::from_std_path(&path).map_err(|err| Error::custom(format!("aip.pdf.page_split failed. {err}")))?;

	// Validate source file exists
	if !pdf_path.exists() {
		return Err(Error::custom(format!("aip.pdf.page_split failed. File not found: {path}")).into());
	}

	// Determine destination directory
	let dest_dir_path = if let Some(dir) = dest_dir {
		SPath::new(dir)
	} else {
		// Default: parent directory + stem as folder name
		let parent = pdf_path.parent().unwrap_or_else(|| SPath::new("."));
		let stem = pdf_path.stem();
		parent.join(stem)
	};

	let created_files = pdf::split_pdf_pages(&pdf_path, &dest_dir_path)
		.map_err(|err| Error::custom(format!("aip.pdf.page_split failed. {err}")))?;

	// Convert to Vec<FileInfo> for Lua
	let file_infos: Vec<FileInfo> = created_files
		.into_iter()
		.map(|full_path| FileInfo::new(runtime.dir_context(), full_path.clone(), &full_path))
		.collect();

	file_infos.into_lua(lua)
}
