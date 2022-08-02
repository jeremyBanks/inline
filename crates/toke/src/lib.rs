//! 🌳 `toke` provides a read-only graph interface for traversing Rust syntax token trees.
#![doc(html_favicon_url = "https://icon.jeremy.ca/toke.png")]
#![doc(html_logo_url = "https://icon.jeremy.ca/toke.png")]
#![cfg_attr(debug_assertions, allow(unused))]

pub(crate) mod compat;
pub(crate) mod sync;

#[doc(hidden)]
mod outer;
#[doc(inline)]
pub use self::outer::*;
