//! `toke` provides a read-only graph interface for traversing Rust syntax files.
#![doc(html_favicon_url = "https://static.jeremy.ca/toke.png")]
#![doc(html_logo_url = "https://static.jeremy.ca/toke.png")]
#![cfg_attr(debug_assertions, allow(unused))]

pub(crate) mod inner;

#[doc(hidden)]
mod outer;
#[doc(inline)]
pub use self::outer::*;
