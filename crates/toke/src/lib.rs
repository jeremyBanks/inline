//! 🌲 `toke` provides a read-only graph interface for traversing Rust syntax token trees.
#![doc(
    html_favicon_url = r#"data:image/svg+xml,<?xml version="1.0" encoding="UTF-8"?><svg width="24" height="24" xmlns="http://www.w3.org/2000/svg"><text text-anchor="middle" x="12" y="19" font-size="20">🌲</text></svg>"#,
    html_logo_url = r#"data:image/svg+xml,<?xml version="1.0" encoding="UTF-8"?><svg width="24" height="24" xmlns="http://www.w3.org/2000/svg"><text text-anchor="middle" x="12" y="19" font-size="20">🌲</text></svg>"#
)]
#![cfg_attr(debug_assertions, allow(unused))]

pub(crate) mod debug;
pub(crate) mod internal;

#[doc(inline)]
pub use self::common_types::*;
#[doc(inline)]
pub use self::outer::*;

#[doc(hidden)]
mod common_types;
#[doc(hidden)]
mod outer;
