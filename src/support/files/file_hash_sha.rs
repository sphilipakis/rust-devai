use crate::Result;
use base64::Engine as _;
use base64::engine::general_purpose;
use sha2::{Digest, Sha256, Sha512};
use simple_fs::SPath;
use std::fs::File;
use std::io::{BufReader, Read};

// region:    --- Sha256

pub fn hash_file_sha256_hex(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file_sha256(path)?;
	Ok(hex::encode(hash))
}

pub fn hash_file_sha256_b58(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file_sha256(path)?;
	Ok(bs58::encode(hash).into_string())
}

pub fn hash_file_sha256_b64(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file_sha256(path)?;
	Ok(general_purpose::STANDARD.encode(hash))
}

pub fn hash_file_sha256_b64u(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file_sha256(path)?;
	Ok(general_purpose::URL_SAFE_NO_PAD.encode(hash))
}

// endregion: --- Sha256

// region:    --- Sha512

pub fn hash_file_sha512_hex(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file_sha512(path)?;
	Ok(hex::encode(hash))
}

pub fn hash_file_sha512_b58(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file_sha512(path)?;
	Ok(bs58::encode(hash).into_string())
}

pub fn hash_file_sha512_b64(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file_sha512(path)?;
	Ok(general_purpose::STANDARD.encode(hash))
}

pub fn hash_file_sha512_b64u(path: impl AsRef<SPath>) -> Result<String> {
	let hash = hash_file_sha512(path)?;
	Ok(general_purpose::URL_SAFE_NO_PAD.encode(hash))
}

// endregion: --- Sha512

// region:    --- Support

fn hash_file_sha256(path: impl AsRef<SPath>) -> Result<Vec<u8>> {
	let path = path.as_ref();
	let file = File::open(path)?;
	let mut reader = BufReader::new(file);
	let mut hasher = Sha256::new();
	let mut buffer = [0; 8192]; // 8KB buffer

	loop {
		let n = reader.read(&mut buffer)?;
		if n == 0 {
			break;
		}
		hasher.update(&buffer[..n]);
	}

	let hash_bytes = hasher.finalize().to_vec();
	Ok(hash_bytes)
}

fn hash_file_sha512(path: impl AsRef<SPath>) -> Result<Vec<u8>> {
	let path = path.as_ref();
	let file = File::open(path)?;
	let mut reader = BufReader::new(file);
	let mut hasher = Sha512::new();
	let mut buffer = [0; 8192]; // 8KB buffer

	loop {
		let n = reader.read(&mut buffer)?;
		if n == 0 {
			break;
		}
		hasher.update(&buffer[..n]);
	}

	let hash_bytes = hasher.finalize().to_vec();
	Ok(hash_bytes)
}

// endregion: --- Support
