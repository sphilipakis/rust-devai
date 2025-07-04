use crate::dir_context::PackDir;
use crossterm::{
	execute,
	style::{Attribute, Print, ResetColor, SetAttribute},
};
use std::{collections::HashSet, io::stdout};

#[allow(unused_must_use)] // TODO: need to remove and make this function return error
pub fn print_pack_list(pack_dirs: &[&PackDir], _interactive: bool) {
	let mut stdout = stdout();

	let mut width = 0;
	for pack_dir in pack_dirs.iter() {
		width = width.max(pack_dir.namespace.len() + pack_dir.name.len());
	}
	width += 5;

	let mut existing_set: HashSet<String> = HashSet::new();

	// (active, pack_ref, pretty_path)
	let data: Vec<(bool, String, String)> = pack_dirs
		.iter()
		.map(|p| {
			let pack_ref = p.to_string();
			let active = if existing_set.contains(&pack_ref) {
				false
			} else {
				existing_set.insert(pack_ref.to_string());
				true
			};
			(active, pack_ref, p.pretty_path())
		})
		.collect::<Vec<_>>();

	execute!(stdout, Print("\nListing all available aipacks:\n\n"));

	for (active, name, path) in data.iter() {
		let (bullet, weight_ref, weight_path) = if *active {
			("•", Attribute::Bold, Attribute::Reset)
		} else {
			("-", Attribute::Dim, Attribute::Dim)
		};
		execute!(
			stdout,
			SetAttribute(weight_ref),
			Print(format!("{bullet} {name:<width$}")),
			ResetColor,
			SetAttribute(weight_path),
			Print(format!("- {path}\n")),
			ResetColor,
			SetAttribute(Attribute::Reset)
		);
	}
}
