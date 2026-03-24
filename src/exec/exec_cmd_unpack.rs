use crate::Result;
use crate::dir_context::DirContext;
use crate::exec::cli::UnpackArgs;
use crate::exec::packer::{UnpackedPack, unpack_pack};
use crate::hub::get_hub;

/// Executes the unpack command which unpacks a repo pack into the workspace custom pack area
pub async fn exec_unpack(dir_context: DirContext, unpack_args: UnpackArgs) -> Result<UnpackedPack> {
	let hub = get_hub();

	hub.publish(format!(
		"\n==== Unpacking pack:\n\n{:>15} {}",
		"Pack Ref:", unpack_args.pack_ref
	))
	.await;

	let unpacked = unpack_pack(&dir_context, &unpack_args.pack_ref, unpack_args.force).await?;

	let source_label = match unpacked.source.as_str() {
		"installed" => "installed copy",
		"remote" => "downloaded from repo",
		other => other,
	};

	hub.publish(format!(
		"{:>15} {}\n{:>15} {}\n{:>15} {}",
		"Source:", source_label, "Namespace:", unpacked.namespace, "Name:", unpacked.name,
	))
	.await;

	hub.publish(format!("{:>15} {}", "Unpacked To:", unpacked.dest_path)).await;

	hub.publish(
		"\nNote: Workspace custom packs take precedence over installed packs.\n      Running this pack will now use the unpacked workspace version."
			.to_string(),
	)
	.await;

	hub.publish("\n==== DONE".to_string()).await;

	Ok(unpacked)
}
