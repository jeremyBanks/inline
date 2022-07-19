#![doc = include_str!("./pre-readme.md")]
//!
#![doc = include_str!("../README.md")]
//!
#![doc = document_features::document_features!()]
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
#![cfg_attr(doc, feature(doc_auto_cfg, doc_cfg))]

mod features;
pub(crate) use self::features::*;

mod util;
pub(crate) use self::util::*;

mod literal;
pub use self::literal::*;

mod litter;
pub use self::litter::*;

mod literal_ext;
pub use self::literal_ext::*;

mod assertions;
pub use self::assertions::assert_eq;

mod litter_index;
pub use self::litter_index::*;

mod litter_handle;
pub use self::litter_handle::*;

mod serde;
pub use self::serde::*;
