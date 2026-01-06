//! Self-modifying values that update their source code at runtime.
//!
//! `code-cell` provides smart pointers that can modify their own source code.
//! This is an experimental approach to snapshot testing and self-modifying code.
//!
//! # Example
//!
//! ```no_run
//! use code_cell::code_cell;
//!
//! let mut counter = code_cell(0u32);
//! println!("Run #{}", *counter + 1);
//! let current = *counter;
//! counter.value = current + 1;
//! // In Write mode, the source file is updated with the new value
//! ```
//!
//! # Modes
//!
//! CodeCell has four modes controlled by the `CODE_CELL_MODE` environment variable:
//!
//! - **Write** (default outside tests): Changes are written to source files
//! - **Verify** (default in tests): Validates values match the source
//! - **Memory**: Changes in memory only, no file writes
//! - **Reject**: Rejects any write attempts
//!
//! ```bash
//! CODE_CELL_MODE=write cargo test    # Update all snapshots
//! cargo test                          # Verify snapshots (default)
//! ```
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
//! 1. The `code_cell()` function captures the source location via `#[track_caller]`
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
pub mod runtime;
pub mod registry;

pub use value::*;
pub use inline::*;
pub use runtime::*;
pub use ext::*;
pub use flush::{flush_all, start_background_flush};
pub use dirty::{has_dirty_literals, dirty_count};
