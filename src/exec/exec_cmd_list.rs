use crate::Result;
use crate::cli::ListArgs;
use crate::dir_context::{DirContext, lookup_pack_dirs};
use crate::hub::get_hub;

pub async fn exec_list(dir_context: DirContext, list_args: ListArgs) -> Result<()> {
	// -- extract the optional namespace / pack_name from the args
	// if no, @, then, assume it is the namespace
	// TODO: Handle the case where we have some special char in namespace
	let (namespace, pack_name) = if let Some(pack_ref) = list_args.pack_ref.as_deref() {
		let (namespace, pack_name) = pack_ref.split_once('@').unwrap_or(("pack_ref", ""));
		let namespace = if namespace.is_empty() { None } else { Some(namespace) };
		let pack_name = if pack_name.is_empty() { None } else { Some(pack_name) };
		(namespace, pack_name)
	} else {
		(None, None)
	};

	let pack_dirs = lookup_pack_dirs(&dir_context, namespace, pack_name)?;

	get_hub().publish(pack_dirs).await;

	Ok(())
}
