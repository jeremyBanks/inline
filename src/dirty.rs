//! Dirty cell tracking for background flushing.
//!
//! Tracks which cells have been modified but not yet written to disk.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Key identifying a unique cell location
type DirtyKey = (PathBuf, u32, u32); // (file, line, column)

/// Global set of dirty cell locations
static DIRTY_CELLS: Lazy<Mutex<HashSet<DirtyKey>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

/// Mark a cell as dirty (modified but not written)
pub(crate) fn mark_dirty(file: &Path, line: u32, column: u32) {
    let key = (file.to_path_buf(), line, column);
    DIRTY_CELLS.lock().insert(key);
}

/// Clear the dirty flag for a cell (after successful write)
pub(crate) fn clear_dirty(file: &Path, line: u32, column: u32) {
    let key = (file.to_path_buf(), line, column);
    DIRTY_CELLS.lock().remove(&key);
}

/// Check if there are any dirty cells
pub fn has_dirty_cells() -> bool {
    !DIRTY_CELLS.lock().is_empty()
}

/// Get the count of dirty cells
pub fn dirty_count() -> usize {
    DIRTY_CELLS.lock().len()
}

/// Get a snapshot of all dirty cell locations (for flush_all)
pub(crate) fn get_dirty_cells() -> Vec<DirtyKey> {
    DIRTY_CELLS.lock().iter().cloned().collect()
}

/// Clear all dirty flags (used after successful flush_all)
pub(crate) fn clear_all_dirty() {
    DIRTY_CELLS.lock().clear();
}
