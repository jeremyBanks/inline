#![doc = include_str!("./pre-readme.md")]
//!
#![doc = include_str!("../README.md")]
//!
#![doc = document_features::document_features!()]
// We wan to warn about these...
#![warn(
    unused_qualifications,
    unused_macro_rules,
    unused_import_braces,
    unused_lifetimes,
    unused_crate_dependencies,
    missing_docs
)]
// ...but not in debug builds or the IDE, that's too noisy.
// So we skip them except in test, doc, and release builds.
#![cfg_attr(
    all(debug_assertions, not(all(any(test, doc), not(feature = "__all__")))),
    allow(
        unused,
        unused_crate_dependencies,
        missing_docs,
        unused_import_braces,
        unused_macro_rules,
        unused_qualifications
    )
)]
// Note that Cargo ignores all of these built-in lints for dependencies, and
// this crate is a library, not a binary. So although we promote warnings
// to errors, this is only relevant for when we build or run this as
// `--release` directly, during development and testing, not for our users.
#![cfg_attr(not(debug_assertions), deny(warnings))]

mod literal;
pub use self::literal::*;

mod litter;
pub use self::litter::*;

mod literal_ext;
pub use self::literal_ext::*;

mod asserts;
pub use self::asserts::*;

#[cfg(feature = "std")]
mod litter_index;
#[cfg(feature = "std")]
pub use self::litter_index::*;

#[cfg(feature = "std")]
mod litter_handle;
#[cfg(feature = "std")]
pub use self::litter_handle::*;

#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "serde")]
pub use self::serde::*;
