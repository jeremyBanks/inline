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
// So we skip them except in test and release builds.
#![cfg_attr(
    all(debug_assertions, not(feature = "__all__")),
    allow(
        unused,
        unused_crate_dependencies,
        missing_docs,
        unused_import_braces,
        unused_macro_rules,
        unused_qualifications
    )
)]

mod features;

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
