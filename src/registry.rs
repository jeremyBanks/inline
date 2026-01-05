//! Type-erased value registry for static persistence of literal values.
//!
//! This module provides a global registry that allows literal values to persist
//! across function calls within the same execution. Each unique source location
//! (file, stable_index) and type gets exactly one shared value that lives for
//! the entire program lifetime.
//!
//! # Registry Key Stability
//!
//! The registry uses **index-based keys** `(file, stable_index, TypeId)` where
//! `stable_index` is the Nth literal in the file (0, 1, 2, ...). This index
//! remains constant even when lines are inserted above the literal, enabling
//! values to persist across source code edits.
//!
//! The stable index is resolved from compile-time `(line, column)` coordinates
//! on first access to each file, with efficient caching to avoid repeated parsing.
//! Within a single execution, this resolution happens at most once per file.
//!
//! # Implementation
//!
//! Uses a global `HashMap` storing raw pointers to `Box<Mutex<LiteralInner<T>>>`.
//! The boxes are intentionally leaked to provide true `'static` lifetime.
//! Type safety is ensured by including `TypeId` in the registry key.
//!
//! # Safety
//!
//! The unsafe pointer casting is safe because:
//! - Pointers are stored in a static registry and never freed (intentional leak)
//! - `TypeId` in the key guarantees we only cast to the correct type
//! - Boxes are allocated by this module, pointers are valid for `'static`

use crate::inline::LiteralInner;
use crate::literal::Value;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::any::TypeId;
use std::collections::HashMap;
use std::path::PathBuf;

/// Registry key that can represent either a stable index or a (line, column) position
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IndexOrPosition {
    /// Stable index (Nth literal in file) - preferred when file exists
    Index(usize),
    /// Fallback (line, column) position - used when file doesn't exist
    Position(u32, u32),
}

/// Type alias for the registry key: (file, index_or_position, type_id)
///
/// Prefers stable index when the file exists, which remains constant even when
/// lines are inserted above the literal. Falls back to (line, column) position
/// when the file doesn't exist (e.g., for testing or compiled binaries).
type RegistryKey = (PathBuf, IndexOrPosition, TypeId);

/// Type alias for the registry value: raw pointer as usize
type RegistryValue = usize;

/// Global registry mapping (file, index_or_position, type) to raw pointers.
///
/// Each entry is a `Box<Mutex<LiteralInner<T>>>` cast to `usize` for type erasure.
/// The TypeId in the key ensures type safety when casting back.
/// Uses stable index when file exists (values persist across line insertions),
/// or (line, column) as fallback when file doesn't exist (for testing).
static VALUE_REGISTRY: Lazy<Mutex<HashMap<RegistryKey, RegistryValue>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get or create a static literal value at the given source location.
///
/// Resolves the stable index from (line, column) on first access to a file,
/// with efficient caching to avoid repeated parsing. The stable index ensures
/// values persist even when lines are inserted above the literal.
///
/// **Note:** This is an internal function called by the `literal!` macro.
/// Users should use the macro instead.
#[doc(hidden)]
pub fn get_or_create<T: Value + 'static>(
    initial: T,
    file: &'static str,
    line: u32,
    column: u32,
) -> &'static Mutex<LiteralInner<T>> {
    // Try to resolve the stable index from (line, column)
    // This parses the file once per file and caches the (line, column) → index mapping
    // If the file doesn't exist (e.g., in tests or compiled binaries), fall back to (line, column)
    let path = PathBuf::from(file);
    let index_or_position = match crate::runtime::get_macro_index(&path, line, column) {
        Ok(index) => IndexOrPosition::Index(index),
        Err(_) => {
            // File doesn't exist or can't be parsed - use (line, column) as fallback
            // This allows literals to work in test scenarios with non-existent files
            IndexOrPosition::Position(line, column)
        }
    };

    // Build the registry key
    // Prefers stable index for files that exist, falls back to (line, column) otherwise
    let key = (path, index_or_position, TypeId::of::<LiteralInner<T>>());

    // Get or create the raw pointer in the registry
    let ptr_as_usize = {
        let mut registry = VALUE_REGISTRY.lock();
        *registry.entry(key).or_insert_with(|| {
            // Create a new boxed value and leak it for 'static lifetime
            let mut inner = LiteralInner::new(initial.clone(), file, line, column);

            // In Verify mode, verify the initial value matches source on first access
            if crate::runtime::get_mode() == crate::runtime::Mode::Verify {
                // Try to resolve index and verify
                if inner.resolve_index().is_ok() {
                    if let Err(e) = inner.verify_source(&initial) {
                        panic!(
                            "Initial literal value verification failed at {}:{}:{}\n\
                             The value provided to literal!() doesn't match the source file.\n\
                             {}",
                            file, line, column, e
                        );
                    }
                }
                // If index resolution fails, we can't verify - this is ok for Memory mode fallback
            }

            let boxed = Box::new(Mutex::new(inner));
            Box::into_raw(boxed) as usize
        })
    };

    // SAFETY:
    // - Pointer stored in static registry, never freed (intentional leak)
    // - TypeId in key guarantees we only cast to the correct type T
    // - Box was allocated above, pointer is valid for 'static
    // - Multiple threads can safely share the &'static reference
    unsafe { &*(ptr_as_usize as *const Mutex<LiteralInner<T>>) }
}
