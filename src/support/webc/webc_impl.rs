//! Module to interact with web resource
//! NOTE: Right now, this is not used for the aip_web (should be)
//! NOTE: Right now, this is just one function, stream download file (used for install pack, update)
//!

use crate::{Error, Result};
use reqwest::Client;
use simple_fs::{SPath, ensure_file_dir};

pub async fn web_download_to_file(url: &str, dest_file: &SPath) -> Result<()> {
	let client = Client::new();
	let response = client.get(url).send().await.map_err(|e| Error::FailToDownload {
		url: url.to_string(),
		dest_file: dest_file.to_string(),
		cause: e.to_string(),
	})?;

	// Check if the request was successful
	if !response.status().is_success() {
		return Err(Error::FailToDownload {
			url: url.to_string(),
			dest_file: dest_file.to_string(),
			cause: format!("httpcode code is: {}", response.status()),
		});
	}

	ensure_file_dir(dest_file)?;

	// Stream the response body to file
	let mut stream = response.bytes_stream();
	use tokio::fs::File as TokioFile;
	use tokio::io::AsyncWriteExt;

	// We need to use tokio's async file for proper streaming
	let mut file = TokioFile::create(&dest_file).await.map_err(|e| Error::FailToDownload {
		url: url.to_string(),
		dest_file: dest_file.to_string(),
		cause: e.to_string(),
	})?;

	while let Some(chunk_result) = tokio_stream::StreamExt::next(&mut stream).await {
		let chunk = chunk_result.map_err(|e| Error::FailToDownload {
			url: url.to_string(),
			dest_file: dest_file.to_string(),
			cause: e.to_string(),
		})?;

		file.write_all(&chunk).await.map_err(|e| Error::FailToDownload {
			url: url.to_string(),
			dest_file: dest_file.to_string(),
			cause: e.to_string(),
		})?;
	}

	file.flush().await.map_err(|e| Error::FailToDownload {
		url: url.to_string(),
		dest_file: dest_file.to_string(),
		cause: e.to_string(),
	})?;

	Ok(())
}
