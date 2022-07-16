#![doc = include_str!("./pre-readme.md")]
//!
#![doc = include_str!("../README.md")]
//!
#![doc = document_features::document_features!()]
#![cfg_attr(debug_assertions, allow(unused))]

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

#[cfg(feature = "rkyv")]
mod rkyv;
#[cfg(feature = "rkyv")]
pub use self::rkyv::*;
