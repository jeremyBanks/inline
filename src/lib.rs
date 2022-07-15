//! [`litter::Litter`]: crate::Litter
//! [`litter::Literal`]: crate::Literal
//! [`litter::LiteralExt`]: crate::LiteralExt
//!
#![doc = include_str!("../README.md")]
//!
#![doc = ::document_features::document_features!()]

mod literal;
pub use self::literal::*;

mod litter;
pub use self::litter::*;

// mod asserts;
// mod runtime;
// test

// #[cfg(feature = "json")]
// mod json;

// #[cfg(feature = "json")]
// pub use json::*;

// #[cfg(feature = "yaml")]
// mod yaml;

// #[cfg(feature = "yaml")]
// pub use yaml::*;

// #[cfg(feature = "toml")]
// mod toml;

// #[cfg(feature = "toml")]
// pub use toml::*;

// pub use asserts::*;
// pub use literal::*;
// pub use litter::*;
// pub use runtime::*;
