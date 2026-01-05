//! Type-erased value registry for static persistence of literal values.
//!
//! This module provides a global registry that allows literal values to persist
//! across function calls within the same execution. Each unique source location
//! (file, line, column) and type gets exactly one shared value that lives for
//! the entire program lifetime.
//!
//! # Registry Key Stability
//!
//! The registry uses **(line, column)-based keys** `(file, line, column, TypeId)` where
//! (line, column) come from compile-time `file!()`, `line!()`, `column!()` macros.
//! These coordinates never change even when code is edited elsewhere.
//!
//! The stable index (Nth literal in file) is resolved lazily only when writing/verifying,
//! not during registry lookup. This eliminates file I/O on reads.
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

/// Type alias for the registry key: (file, line, column, type_id)
///
/// Uses compile-time (line, column) from file!(), line!(), column!() which never change.
/// The stable index is resolved lazily only when writing/verifying.
type RegistryKey = (PathBuf, u32, u32, TypeId);

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
/// Uses compile-time (file, line, column) as the registry key. No file I/O occurs
/// during this call - the stable index is resolved lazily only when writing/verifying.
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
    // Build the registry key using compile-time (line, column)
    // No file parsing needed - these coordinates never change
    let key = (
        PathBuf::from(file),
        line,
        column,
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
