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
//! - `inline::cell_default::<T>()` - with default value
//! - `inline::var(value)` - alias
//! - `inline::snapshot(value)` - alias
//! - `inline::HACK(value)` - alias (playful placeholder)
//!
//! Macro versions: `cell!()`, `cell_default!()`
//!
//! ## One-shot replacement (canonical: `replace`)
//!
//! Returns `T` directly, replacing the entire call with the baked value.
//!
//! - `inline::replace(value)` - canonical
//! - `inline::replace_default::<T>()` - with default value
//! - `inline::val(value)` - alias
//! - `inline::eval(value)` - alias
//! - `inline::REPLACE_ME(value)` - alias (playful placeholder)
//!
//! Macro versions: `replace!()`, `replace_default!()`
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
//! 1. Functions capture the source location via `#[track_caller]`
//! 2. Mutations are detected on drop (comparing original vs current value)
//! 3. The source file is parsed and the call is located by stable index
//! 4. Character-range splicing replaces the appropriate part:
//!    - **Function calls**: the last argument (trailing position for extensibility)
//!    - **Method calls**: the receiver expression
//!    - **Macros**: entire contents inside delimiters
//! 5. Original formatting is preserved
//!
//! For `replace()` mode, the entire call/macro expression is replaced.

// Compile-time check: write and no-write features are mutually exclusive
#[cfg(all(feature = "write", feature = "no-write"))]
compile_error!("Features 'write' and 'no-write' are mutually exclusive. Enable only one.");

mod value;
mod inline;
mod dirty;
mod ext;
mod flush;
mod replace;
mod tokens;
pub mod runtime;
pub mod registry;

pub use value::*;
pub use inline::*;
pub use runtime::*;
pub use ext::*;
pub use flush::{flush_all, start_background_flush};
pub use dirty::{has_dirty_cells, dirty_count};
pub use tokens::Tokens;

// Re-export replace functions and aliases
pub use replace::{replace, replace_at, replace_default, val, eval, REPLACE_ME};

// Re-export quote for macro users
pub use quote::quote;

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

/// Macro version of [`cell_default()`].
///
/// Creates a self-modifying value initialized with the type's default.
/// Identical to calling the `cell_default()` function directly.
///
/// # Example
///
/// ```no_run
/// let mut counter = inline::cell_default!(u32);
/// *counter += 1;
/// ```
#[macro_export]
macro_rules! cell_default {
    ($type:ty) => {
        $crate::cell_default::<$type>()
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

/// Macro version of [`replace_default()`].
///
/// One-shot code generation initialized with the type's default value.
/// Identical to calling the `replace_default()` function directly.
///
/// # Example
///
/// ```no_run
/// let config = inline::replace_default!(Vec<String>);
/// ```
#[macro_export]
macro_rules! replace_default {
    ($type:ty) => {
        $crate::replace_default::<$type>()
    };
}

/// Create a self-modifying [`InlineCell`] containing arbitrary tokens.
///
/// The [`Tokens`] type implements [`Bake`](databake::Bake) to produce a macro call
/// that reproduces the original tokens. This macro wraps the tokens in an
/// `InlineCell` for automatic source code updates.
///
/// # Example
///
/// ```no_run
/// use inline::tokens;
///
/// let mut toks = tokens!(foo bar 123 "hello");
///
/// // Mutate the tokens
/// toks.value = inline::Tokens::from_str("new tokens here");
///
/// // On drop, source updates to: tokens!(new tokens here)
/// ```
///
/// For a raw `Tokens` value without the `InlineCell` wrapper, use
/// [`Tokens::from_str()`] directly.
#[macro_export]
macro_rules! tokens {
    ($($tt:tt)*) => {
        $crate::cell($crate::Tokens::from_str(stringify!($($tt)*)))
    };
}
