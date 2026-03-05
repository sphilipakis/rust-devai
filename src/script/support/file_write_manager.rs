use dashmap::DashMap;
use simple_fs::SPath;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

/// Shared process-level manager for per-file write serialization.
pub struct FileWriteManager {
	locks: DashMap<String, Arc<Mutex<()>>>,
}

impl FileWriteManager {
	pub fn global() -> &'static Self {
		static INSTANCE: LazyLock<FileWriteManager> = LazyLock::new(|| FileWriteManager { locks: DashMap::new() });
		&INSTANCE
	}

	/// Returns a lock handle for the given path. The caller should call `.lock()`
	/// on the returned handle and hold the resulting guard for the duration of
	/// the critical section.
	///
	/// Usage:
	/// ```ignore
	/// let handle = file_write_manager.lock_for_path(&full_path);
	/// let _guard = handle.lock();
	/// // ... perform file write ...
	/// // guard is dropped here, releasing the lock
	/// ```
	pub fn lock_for_path(&self, path: &SPath) -> FilePathLockHandle {
		let lock_key = Self::resolve_lock_key(path);

		let lock_arc = self.locks.entry(lock_key).or_insert_with(|| Arc::new(Mutex::new(()))).clone();

		FilePathLockHandle { mutex: lock_arc }
	}

	fn resolve_lock_key(path: &SPath) -> String {
		// TODO: Later we will need to do absolute to be more accurate
		//       The trick, is that assume the path exist
		// if let Ok(canonical) = path.canonicalize() {
		// 	return canonical.to_string();
		// }

		path.to_string()
	}
}

/// A handle to a per-file lock. Call `.lock()` to acquire the mutex guard.
/// The returned `MutexGuard` borrows from this handle, so the handle must
/// outlive the guard (which is naturally enforced by Rust's borrow checker).
pub struct FilePathLockHandle {
	mutex: Arc<Mutex<()>>,
}

impl FilePathLockHandle {
	pub fn lock(&self) -> MutexGuard<'_, ()> {
		self.mutex.lock().unwrap_or_else(|e| e.into_inner())
	}
}
