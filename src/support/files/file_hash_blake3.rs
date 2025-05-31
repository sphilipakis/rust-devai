use crate::Result;
use base64::Engine as _;
use base64::engine::general_purpose;
use blake3::Hasher;
use simple_fs::SPath;
use std::fs::File;
use std::io::{self, BufReader};

pub fn hash_file_hex(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file(path)?;

	Ok(hex::encode(hash))
}

pub fn hash_file_b58(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file(path)?;
	Ok(bs58::encode(hash).into_string())
}

pub fn hash_file_b64(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file(path)?;
	Ok(general_purpose::STANDARD.encode(hash))
}

pub fn hash_file_b64u(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file(path)?;
	Ok(general_purpose::URL_SAFE_NO_PAD.encode(hash))
}

// region:    --- Support

fn hash_file(path: impl AsRef<SPath>) -> Result<Vec<u8>> {
	let path = path.as_ref();
	// Open the file
	let file = File::open(path)?;

	// Create a buffered reader for efficient reading
	let mut reader = BufReader::new(file);

	let mut hasher = Hasher::new();

	// Use io::copy to pipe the reader directly into the hasher (which implements io::Write)
	io::copy(&mut reader, &mut hasher)?;

	// Finalize the hash
	let hash = hasher.finalize();

	let hash: Vec<u8> = hash.as_bytes().to_vec();

	Ok(hash)
}

// endregion: --- Support
