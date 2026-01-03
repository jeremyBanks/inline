//! Type-erased value registry for static persistence of inline values.
//!
//! This module provides a global registry that allows inline values to persist
//! across function calls within the same execution. Each unique source location
//! (file, line, column) and type gets exactly one shared value that lives for
//! the entire program lifetime.
//!
//! # Implementation
//!
//! Uses a global `HashMap` storing raw pointers to `Box<Mutex<Inline<T>>>`.
//! The boxes are intentionally leaked to provide true `'static` lifetime.
//! Type safety is ensured by including `TypeId` in the registry key.
//!
//! # Safety
//!
//! The unsafe pointer casting is safe because:
//! - Pointers are stored in a static registry and never freed (intentional leak)
//! - `TypeId` in the key guarantees we only cast to the correct type
//! - Boxes are allocated by this module, pointers are valid for `'static`

use crate::{Inline, Literal};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::any::TypeId;
use std::collections::HashMap;
use std::path::PathBuf;

/// Global registry mapping (file, line, column, type) to raw pointers.
///
/// Each entry is a `Box<Mutex<Inline<T>>>` cast to `usize` for type erasure.
/// The TypeId in the key ensures type safety when casting back.
static VALUE_REGISTRY: Lazy<Mutex<HashMap<(PathBuf, u32, u32, TypeId), usize>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get or create a static inline value at the given source location.
///
/// This function returns a `'static` reference to a `Mutex<Inline<T>>` that
/// persists for the entire program lifetime. Multiple calls from the same
/// source location (and with the same type) will return the same shared value.
///
/// # Arguments
///
/// * `initial` - The initial value to use if creating a new entry
/// * `file` - Source file path (from `file!()` macro)
/// * `line` - Line number (from `line!()` macro)
/// * `column` - Column number (from `column!()` macro)
///
/// # Returns
///
/// A `'static` reference to a `Mutex<Inline<T>>`. The same reference is
/// returned for all calls with the same source location and type.
///
/// # Example
///
/// ```no_run
/// use inline::registry::get_or_create;
///
/// let value = get_or_create(42u32, file!(), line!(), column!());
/// value.lock().set(100);
/// // Later calls from the same location will see 100
/// ```
pub fn get_or_create<T: Literal + 'static>(
    initial: T,
    file: &'static str,
    line: u32,
    column: u32,
) -> &'static Mutex<Inline<T>> {
    // Build the registry key including TypeId for type safety
    let key = (
        PathBuf::from(file),
        line,
        column,
        TypeId::of::<Inline<T>>(),
    );

    // Get or create the raw pointer in the registry
    let ptr_as_usize = {
        let mut registry = VALUE_REGISTRY.lock();
        *registry.entry(key).or_insert_with(|| {
            // Create a new boxed value and leak it for 'static lifetime
            let boxed = Box::new(Mutex::new(Inline::__new(initial, file, line, column)));
            Box::into_raw(boxed) as usize
        })
    };

    // SAFETY:
    // - Pointer stored in static registry, never freed (intentional leak)
    // - TypeId in key guarantees we only cast to the correct type T
    // - Box was allocated above, pointer is valid for 'static
    // - Multiple threads can safely share the &'static reference
    unsafe { &*(ptr_as_usize as *const Mutex<Inline<T>>) }
}
