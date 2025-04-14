use crate::Result;
use crate::cli::XelfUpdateArgs;
use crate::hub::get_hub;

pub async fn exec_xelf_update(_args: XelfUpdateArgs) -> Result<()> {
	get_hub()
		.publish(
			r#"
To update AIPACK, just install the latest version from

https://aipack.ai/doc/install

(Future versions of AIPACK will support full self-update)
		"#,
		)
		.await;

	Ok(())
}
