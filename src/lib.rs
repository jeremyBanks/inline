//! Self-modifying values that update their source code at runtime.
//!
//! `inline` provides smart pointers that can modify their own source code.
//! This is an experimental approach to snapshot testing and self-modifying code.
//!
//! # Example
//!
//! ```no_run
//! use inline::cell;
//!
//! let mut counter = cell(0u32);
//! println!("Run #{}", *counter + 1);
//! let current = *counter;
//! counter.value = current + 1;
//! // In Write mode, the source file is updated with the new value
//! ```
//!
//! # Modes
//!
//! InlineCell has four modes controlled by the `INLINE_MODE` environment variable:
//!
//! - **Write** (default outside tests): Changes are written to source files
//! - **Verify** (default in tests): Validates values match the source
//! - **Memory**: Changes in memory only, no file writes
//! - **Reject**: Rejects any write attempts
//!
//! ```bash
//! INLINE_MODE=write cargo test    # Update all snapshots
//! cargo test                       # Verify snapshots (default)
//! ```
//!
//! # Functions
//!
//! ## Mutable cell (canonical: `cell`)
//!
//! Returns an `InlineCell<T>` that persists mutations to source code.
//!
//! - `inline::cell(value)` - canonical
//! - `inline::var(value)` - alias
//! - `inline::snapshot(value)` - alias
//! - `inline::HACK(value)` - alias (playful placeholder)
//!
//! ## One-shot replacement (canonical: `replace`)
//!
//! Returns `T` directly, replacing the entire call with the baked value.
//!
//! - `inline::replace(value)` - canonical
//! - `inline::val(value)` - alias
//! - `inline::eval(value)` - alias
//! - `inline::REPLACE_ME(value)` - alias (playful placeholder)
//!
//! # Supported Types
//!
//! Any type implementing `Bake + Clone + PartialEq` can be used.
//! See the [`databake`](https://docs.rs/databake) crate for types that implement `Bake`.
//!
//! # Safety and Limitations
//!
//! - This is an experimental library
//! - Modifies source files at runtime
//! - Requires running under cargo
//! - Not recommended for production use
//!
//! # How It Works
//!
//! 1. The `cell()` function captures the source location via `#[track_caller]`
//! 2. Mutations are detected on drop (comparing original vs current value)
//! 3. The source file is parsed and the function call is located by stable index
//! 4. Character-range splicing replaces only the call's argument
//! 5. Original formatting is preserved

// Compile-time check: write and no-write features are mutually exclusive
#[cfg(all(feature = "write", feature = "no-write"))]
compile_error!("Features 'write' and 'no-write' are mutually exclusive. Enable only one.");

mod value;
mod inline;
mod dirty;
mod ext;
mod flush;
mod replace;
pub mod runtime;
pub mod registry;

pub use value::*;
pub use inline::*;
pub use runtime::*;
pub use ext::*;
pub use flush::{flush_all, start_background_flush};
pub use dirty::{has_dirty_literals, dirty_count};

// Re-export replace functions and aliases
pub use replace::{replace, replace_at, val, eval, REPLACE_ME};

// =============================================================================
// Macro wrappers
// =============================================================================
// These macros provide an alternative syntax for users who prefer macro invocations.
// They work identically to the function versions - #[track_caller] on the inner
// function captures the macro call site correctly.

/// Macro version of [`cell()`].
///
/// Creates a self-modifying value that can update its source code.
/// Identical to calling the `cell()` function directly.
///
/// # Example
///
/// ```no_run
/// use inline::cell;
///
/// let mut counter = inline::cell!(0u32);
/// *counter += 1;
/// ```
#[macro_export]
macro_rules! cell {
    ($value:expr) => {
        $crate::cell($value)
    };
}

/// Macro version of [`replace()`].
///
/// One-shot code generation that replaces the entire macro invocation
/// with the baked value. Identical to calling the `replace()` function directly.
///
/// # Example
///
/// ```no_run
/// let author = inline::replace!(std::env::var("USER").unwrap_or_default());
/// ```
#[macro_export]
macro_rules! replace {
    ($value:expr) => {
        $crate::replace($value)
    };
}
