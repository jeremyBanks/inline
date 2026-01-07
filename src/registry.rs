//! Type-erased value registry for static persistence of cell values.
//!
//! This module provides a global registry that allows cell values to persist
//! across function calls within the same execution. Each unique source location
//! (file, stable_index) and type gets exactly one shared value that lives for
//! the entire program lifetime.
//!
//! # Registry Key Stability
//!
//! The registry uses **index-based keys** `(file, stable_index, TypeId)` where
//! `stable_index` is the Nth call/macro in the file (0, 1, 2, ...). This index
//! remains constant even when lines are inserted above the call, enabling
//! values to persist across source code edits.
//!
//! The stable index is resolved from compile-time `(line, column)` coordinates
//! on first access to each file, with efficient caching to avoid repeated parsing.
//! Within a single execution, this resolution happens at most once per file.
//!
//! # Implementation
//!
//! Uses a global `HashMap` storing raw pointers to `Box<Mutex<InlineCellInner<T>>>`.
//! The boxes are intentionally leaked to provide true `'static` lifetime.
//! Type safety is ensured by including `TypeId` in the registry key.
//!
//! # Safety
//!
//! The unsafe pointer casting is safe because:
//! - Pointers are stored in a static registry and never freed (intentional leak)
//! - `TypeId` in the key guarantees we only cast to the correct type
//! - Boxes are allocated by this module, pointers are valid for `'static`

use crate::inline::InlineCellInner;
use crate::value::Value;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::any::TypeId;
use std::collections::HashMap;
use std::path::PathBuf;

/// Registry key that can represent either a stable index or a (line, column) position
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IndexOrPosition {
    /// Stable index (Nth call/macro in file) - preferred when file exists
    Index(usize),
    /// Fallback (line, column) position - used when file doesn't exist
    Position(u32, u32),
}

/// Type alias for the registry key: (file, index_or_position, type_id)
///
/// Prefers stable index when the file exists, which remains constant even when
/// lines are inserted above the call. Falls back to (line, column) position
/// when the file doesn't exist (e.g., for testing or compiled binaries).
type RegistryKey = (PathBuf, IndexOrPosition, TypeId);

/// Type alias for the registry value: raw pointer as usize
type RegistryValue = usize;

/// Global registry mapping (file, index_or_position, type) to raw pointers.
///
/// Each entry is a `Box<Mutex<InlineCellInner<T>>>` cast to `usize` for type erasure.
/// The TypeId in the key ensures type safety when casting back.
/// Uses stable index when file exists (values persist across line insertions),
/// or (line, column) as fallback when file doesn't exist (for testing).
static VALUE_REGISTRY: Lazy<Mutex<HashMap<RegistryKey, RegistryValue>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get or create a static cell value at the given source location.
///
/// Resolves the stable index from (line, column) on first access to a file,
/// with efficient caching to avoid repeated parsing. The stable index ensures
/// values persist even when lines are inserted above the call.
///
/// Uses `#[track_caller]` to automatically capture the call site location.
///
/// **Note:** This is an internal function called by `cell()` and `cell_default()`.
/// Users should use those functions instead.
#[doc(hidden)]
#[track_caller]
pub fn get_or_create<T: Value + 'static>(
    initial: T,
) -> &'static Mutex<InlineCellInner<T>> {
    let loc = std::panic::Location::caller();
    get_or_create_at(initial, loc.file(), loc.line(), loc.column())
}

/// Get or create a static cell value at an explicit source location.
///
/// This is the internal implementation that takes explicit location parameters.
/// Used by tests that need to specify synthetic file locations.
#[doc(hidden)]
pub fn get_or_create_at<T: Value + 'static>(
    initial: T,
    file: &str,
    line: u32,
    column: u32,
) -> &'static Mutex<InlineCellInner<T>> {
    // Auto-start background flush thread on first cell access
    let _ = crate::flush::start_background_flush_internal();

    // Try to resolve the stable index from (line, column)
    // This parses the file once per file and caches the (line, column) → index mapping
    // If the file doesn't exist (e.g., in tests or compiled binaries), fall back to (line, column)
    let path = PathBuf::from(file);
    let index_or_position = match crate::runtime::get_macro_index(&path, line, column) {
        Ok(index) => IndexOrPosition::Index(index),
        Err(_) => {
            // File doesn't exist or can't be parsed - use (line, column) as fallback
            // This allows cells to work in test scenarios with non-existent files
            IndexOrPosition::Position(line, column)
        }
    };

    // Build the registry key
    // Prefers stable index for files that exist, falls back to (line, column) otherwise
    let key = (path, index_or_position, TypeId::of::<InlineCellInner<T>>());

    // Get or create the raw pointer in the registry
    let ptr_as_usize = {
        let mut registry = VALUE_REGISTRY.lock();
        *registry.entry(key).or_insert_with(|| {
            // Create a new boxed value and leak it for 'static lifetime
            let inner = InlineCellInner::new(initial.clone(), file, line, column);

            // TODO: Initial value verification disabled due to false positives
            // When databake serializes values like vec![1,2,3], it produces alloc::vec![1,2,3,]
            // which is semantically equivalent but syntactically different from vec![1,2,3]
            // This causes verification to fail even when values match semantically.
            // We only verify mutations (in Drop), not initial values.

            let boxed = Box::new(Mutex::new(inner));
            Box::into_raw(boxed) as usize
        })
    };

    // SAFETY:
    // - Pointer stored in static registry, never freed (intentional leak)
    // - TypeId in key guarantees we only cast to the correct type T
    // - Box was allocated above, pointer is valid for 'static
    // - Multiple threads can safely share the &'static reference
    unsafe { &*(ptr_as_usize as *const Mutex<InlineCellInner<T>>) }
}
