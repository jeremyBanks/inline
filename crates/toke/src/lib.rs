//! `toke` provides a read-only graph interface for traversing Rust syntax files.
#![doc(html_favicon_url = "https://static.jeremy.ca/toke.png")]
#![doc(html_logo_url = "https://static.jeremy.ca/toke.png")]

pub(crate) mod inner;

mod outer;
pub use self::outer::*;

/// Re-exported types from [`miette`][::miette].
pub mod miette {
    pub use miette::{Diagnostic, SourceCode, SpanContents};
}

/// Re-exported types from [`proc_macro2`][::proc_macro2].
pub mod proc_macro2 {
    pub use proc_macro2::{TokenStream, TokenTree};
}
