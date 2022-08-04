#![doc = include_str!("./pre-docs.md")]
//!
#![doc = include_str!("../README.md")]
//!
#![doc = document_features::document_features!()]
#![doc(html_favicon_url = "https://icon.jeremy.ca/litter.png")]
#![doc(html_logo_url = "https://icon.jeremy.ca/litter.png")]
#![cfg_attr(doc, feature(doc_auto_cfg, doc_cfg))]
// We warn to warn about these...
#![warn(
    absolute_paths_not_starting_with_crate,
    clippy::missing_panics_doc,
    missing_debug_implementations,
    missing_docs,
    noop_method_call,
    unreachable_pub,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications
)]
// ...but some can be too noisy in debug builds or the IDE.
// So we skip those except in test and release builds:
#![cfg_attr(
    all(debug_assertions, not(feature = "__all__")),
    allow(
        missing_docs,
        unused_crate_dependencies,
        unused_import_braces,
        unused_macro_rules,
        unused_qualifications,
        unused
    )
)]

pub(crate) mod assertions;
pub(crate) mod features;
pub(crate) mod literal;
pub(crate) mod literal_ext;
pub(crate) mod litter;
pub(crate) mod litter_handle;
pub(crate) mod litter_index;
pub(crate) mod serde;

#[cfg(all(doc, feature = "databake"))]
#[allow(unused)]
#[doc(hidden)]
pub use ::{databake, serde, yoke, zerovec};

pub use self::{
    assertions::assert_eq,
    literal::{AnyLiteral, Literal},
    literal_ext::LiteralExt,
    litter::{AnyLitter, Litter},
    litter_handle::{AnyLitterHandle, LitterHandle},
};
