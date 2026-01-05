//! Dirty literal tracking for background flushing.
//!
//! Tracks which literals have been modified but not yet written to disk.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Key identifying a unique literal location
type DirtyKey = (PathBuf, u32, u32); // (file, line, column)

/// Global set of dirty literal locations
static DIRTY_LITERALS: Lazy<Mutex<HashSet<DirtyKey>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

/// Mark a literal as dirty (modified but not written)
pub(crate) fn mark_dirty(file: &Path, line: u32, column: u32) {
    let key = (file.to_path_buf(), line, column);
    DIRTY_LITERALS.lock().insert(key);
}

/// Clear the dirty flag for a literal (after successful write)
pub(crate) fn clear_dirty(file: &Path, line: u32, column: u32) {
    let key = (file.to_path_buf(), line, column);
    DIRTY_LITERALS.lock().remove(&key);
}

/// Check if there are any dirty literals
pub fn has_dirty_literals() -> bool {
    !DIRTY_LITERALS.lock().is_empty()
}

/// Get the count of dirty literals
pub fn dirty_count() -> usize {
    DIRTY_LITERALS.lock().len()
}

/// Get a snapshot of all dirty literal locations (for flush_all)
pub(crate) fn get_dirty_literals() -> Vec<DirtyKey> {
    DIRTY_LITERALS.lock().iter().cloned().collect()
}

/// Clear all dirty flags (used after successful flush_all)
pub(crate) fn clear_all_dirty() {
    DIRTY_LITERALS.lock().clear();
}
