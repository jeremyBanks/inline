//! Type-erased value registry for static persistence of literal values.
//!
//! This module provides a global registry that allows literal values to persist
//! across function calls within the same execution. Each unique source location
//! (file, index) and type gets exactly one shared value that lives for
//! the entire program lifetime.
//!
//! # Registry Key Stability
//!
//! The registry uses **index-based keys** `(file, index, TypeId)` where `index`
//! is the position of the literal in the file ("Nth literal!() macro").
//! This is stable across line insertions, unlike line/column-based keys.
//!
//! When a literal!() is first accessed with (file, line, column), we:
//! 1. Parse the file to resolve (line, column) → stable index
//! 2. Use (file, index, TypeId) as the registry key
//! 3. Store the index in LiteralInner for future use
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
use std::path::{Path, PathBuf};

/// Type alias for the registry key: (file, index, type_id)
///
/// The index is the stable position of the literal in the file (Nth literal! macro),
/// which remains constant even when lines are inserted or removed above it.
type RegistryKey = (PathBuf, usize, TypeId);

/// Type alias for the registry value: raw pointer as usize
type RegistryValue = usize;

/// Global registry mapping (file, line, column, type) to raw pointers.
///
/// Each entry is a `Box<Mutex<LiteralInner<T>>>` cast to `usize` for type erasure.
/// The TypeId in the key ensures type safety when casting back.
static VALUE_REGISTRY: Lazy<Mutex<HashMap<RegistryKey, RegistryValue>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get or create a static literal value at the given source location.
///
/// This function resolves the stable index for the literal before looking it up
/// in the registry. The index is stable across line insertions, ensuring values
/// persist even when the source code changes.
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
    let path = Path::new(file);

    // Resolve the stable index from (line, column)
    // This ensures the same literal always maps to the same registry entry,
    // even if lines are inserted or deleted above it
    let index = match crate::runtime::get_macro_index(path, line, column) {
        Ok(idx) => idx,
        Err(_) => {
            // File doesn't exist or can't be parsed
            // This is OK for the initial access - we'll create an entry anyway
            // The error will surface later if the user tries to call .set()
            // For now, use a fallback: hash the (line, column) to a pseudo-index
            // This ensures consistent behavior even without file access
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            line.hash(&mut hasher);
            column.hash(&mut hasher);
            hasher.finish() as usize
        }
    };

    // Build the registry key using the stable index
    let key = (
        PathBuf::from(file),
        index,
        TypeId::of::<LiteralInner<T>>(),
    );

    // Get or create the raw pointer in the registry
    let ptr_as_usize = {
        let mut registry = VALUE_REGISTRY.lock();
        *registry.entry(key).or_insert_with(|| {
            // Create a new boxed value and leak it for 'static lifetime
            let boxed = Box::new(Mutex::new(LiteralInner::new(initial, file, line, column)));
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
