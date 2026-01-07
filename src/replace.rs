//! One-shot code generation via `replace()`.
//!
//! Unlike `cell()` which provides ongoing mutable persistence,
//! `replace()` is for one-time code generation: it evaluates an expression,
//! bakes it to source code, and replaces the entire call with the literal value.
//!
//! # Example
//!
//! ```no_run
//! use inline::replace;
//!
//! // First run: computes current time, writes to source, returns value
//! let timestamp = replace(std::time::SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs());
//!
//! // After source replacement, the code becomes:
//! // let timestamp = 1234567890u64;
//! ```
//!
//! # Semantics
//!
//! - **First call**: Evaluates argument, stores in memory, writes to source, returns value
//! - **Subsequent calls (same run)**: Returns clone from memory, ignores argument
//! - **After replacement**: The `replace(...)` call no longer exists in source

use crate::value::Value;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::any::TypeId;
use std::collections::HashMap;
use std::path::PathBuf;

/// Registry key for replace: (file, index_or_position, type_id)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IndexOrPosition {
    Index(usize),
    Position(u32, u32),
}

type RegistryKey = (PathBuf, IndexOrPosition, TypeId);

/// Stored value with write-tracking
struct StoredValue<T> {
    value: T,
    /// Whether we've already written to source (only write once)
    written: bool,
}

/// Global registry for replace values
/// Each entry stores the value and whether it's been written to source
static REPLACE_REGISTRY: Lazy<Mutex<HashMap<RegistryKey, usize>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// =============================================================================
// Canonical function: replace()
// =============================================================================

/// One-shot code generation that replaces the entire call with the baked value.
///
/// On first execution, evaluates the expression, writes it to source code, and
/// returns the value. On subsequent executions within the same run, returns a
/// clone of the persisted value (ignoring the new argument).
///
/// After the source file is modified, the `replace(...)` call no longer exists -
/// it has been replaced with the literal value.
///
/// # Example
///
/// ```no_run
/// use inline::replace;
///
/// // First run: computes and bakes to source
/// let values = replace(vec![1, 2, 3].iter().map(|x| x * 2).collect::<Vec<_>>());
///
/// // Source becomes:
/// // let values = <[_]>::into_vec(Box::new([2i32, 4i32, 6i32]));
/// ```
///
/// # Requirements
///
/// The value type must implement `Value` (which requires `Bake + Clone + PartialEq`).
#[track_caller]
pub fn replace<T: Value + 'static>(value: T) -> T {
    let loc = std::panic::Location::caller();
    replace_at(value, loc.file(), loc.line(), loc.column())
}

/// Internal implementation with explicit location parameters.
///
/// Used for testing with synthetic file locations.
#[doc(hidden)]
pub fn replace_at<T: Value + 'static>(
    value: T,
    file: &str,
    line: u32,
    column: u32,
) -> T {
    // Try to resolve stable index
    let path = PathBuf::from(file);
    let index_or_position = match crate::runtime::get_macro_index(&path, line, column) {
        Ok(index) => IndexOrPosition::Index(index),
        Err(_) => IndexOrPosition::Position(line, column),
    };

    let key = (path.clone(), index_or_position, TypeId::of::<StoredValue<T>>());

    // Check if we already have a value stored
    let ptr_as_usize = {
        let mut registry = REPLACE_REGISTRY.lock();
        if let Some(&ptr) = registry.get(&key) {
            // Already stored - return clone, don't write again
            let stored = unsafe { &*(ptr as *const Mutex<StoredValue<T>>) };
            return stored.lock().value.clone();
        }

        // First time - store the value
        let stored = StoredValue {
            value: value.clone(),
            written: false,
        };
        let boxed = Box::new(Mutex::new(stored));
        let ptr = Box::into_raw(boxed) as usize;
        registry.insert(key, ptr);
        ptr
    };

    // Get stored value reference
    let stored_mutex = unsafe { &*(ptr_as_usize as *const Mutex<StoredValue<T>>) };
    let mut stored = stored_mutex.lock();

    // Trigger source replacement (if we haven't already and mode allows)
    if !stored.written {
        let mode = crate::runtime::get_mode();

        if mode.can_write() && crate::runtime::is_running_under_cargo() {
            // Write the replacement
            if let Ok(index) = crate::runtime::get_macro_index(&path, line, column) {
                // Bake the value
                let env = databake::CrateEnv::default();
                let baked_tokens = value.bake(&env);

                // Replace the entire expression
                if crate::runtime::replace_expression(&path, index, baked_tokens).is_ok() {
                    stored.written = true;
                }
            }
        } else if mode == crate::runtime::Mode::Verify {
            // In verify mode, we could check the source matches, but since replace
            // is designed to be replaced, verification doesn't make as much sense
            // Just mark as "written" to avoid repeated checks
            stored.written = true;
        } else {
            // Memory mode or Reject mode - don't write, just use in-memory value
            stored.written = true;
        }
    }

    stored.value.clone()
}

// =============================================================================
// Aliases for replace()
// =============================================================================

/// Alias of [`replace()`].
#[track_caller]
pub fn val<T: Value + 'static>(value: T) -> T {
    replace(value)
}

/// Alias of [`replace()`].
#[track_caller]
pub fn eval<T: Value + 'static>(value: T) -> T {
    replace(value)
}

/// Alias of [`replace()`].
///
/// A playful, attention-grabbing name - if you leave this in your code,
/// it's a reminder to replace it with a proper value!
#[track_caller]
#[allow(non_snake_case)]
pub fn REPLACE_ME<T: Value + 'static>(value: T) -> T {
    replace(value)
}
