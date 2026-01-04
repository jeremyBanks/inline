//! Self-modifying values that update their source code at runtime.
//!
//! `jeb-literal` provides mutable literals as smart pointers into your source code.
//! This is an experimental approach to snapshot testing and self-modifying code.
//!
//! # Example
//!
//! ```no_run
//! use jeb_literal::literal;
//!
//! let mut counter = literal!(0u32);
//! println!("Run #{}", *counter + 1);
//! let current = *counter;
//! counter.value = current + 1;
//! // In Write mode, the source file is updated with the new value
//! ```
//!
//! # Modes
//!
//! Literal has four modes controlled by the `LITERAL_MODE` environment variable:
//!
//! - **Write** (default outside tests): Changes are written to source files
//! - **Verify** (default in tests): Validates values match the source
//! - **Memory**: Changes in memory only, no file writes
//! - **Reject**: Rejects any write attempts
//!
//! ```bash
//! LITERAL_MODE=write cargo test    # Update all snapshots
//! cargo test                        # Verify snapshots (default)
//! ```
//!
//! # Supported Types
//!
//! Any type implementing `Bake` from the [`databake`](https://docs.rs/databake) crate
//! can be used with literal values.
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
//! 1. The `literal!()` macro captures the source location
//! 2. When `.set()` is called, the source file is parsed
//! 3. Character-range splicing replaces only the macro's value
//! 4. Original formatting is preserved

mod literal;
mod inline;
pub mod runtime;
pub mod registry;

pub use literal::*;
pub use inline::*;
pub use runtime::*;
