//! 🌲 `toke` provides a graph document interface for traversing Rust syntax token trees.
#![doc(
    html_favicon_url = r#"data:image/svg+xml,<?xml version="1.0" encoding="UTF-8"?><svg width="24" height="24" xmlns="http://www.w3.org/2000/svg"><text text-anchor="middle" x="12" y="19" font-size="20">🌲</text></svg>"#,
    html_logo_url = r#"data:image/svg+xml,<?xml version="1.0" encoding="UTF-8"?><svg width="24" height="24" xmlns="http://www.w3.org/2000/svg"><text text-anchor="middle" x="12" y="19" font-size="20">🌲</text></svg>"#
)]
#![cfg_attr(debug_assertions, allow(unused))]

pub(crate) mod debug;
pub(crate) mod internal;

#[doc(inline)]
pub use self::outer::*;
#[doc(inline)]
pub use self::span::*;

#[doc(hidden)]
mod outer;
#[doc(hidden)]
mod span;

pub extern crate miette;
pub extern crate proc_macro;
pub extern crate proc_macro2;

/// The variants of [`proc_macro::TokenTree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenType {
    Group,
    Ident,
    Punct,
    Literal,
}

impl From<&proc_macro::TokenTree> for TokenType {
    fn from(token: &proc_macro::TokenTree) -> Self {
        match token {
            proc_macro::TokenTree::Group(..) => TokenType::Group,
            proc_macro::TokenTree::Ident(..) => TokenType::Ident,
            proc_macro::TokenTree::Punct(..) => TokenType::Punct,
            proc_macro::TokenTree::Literal(..) => TokenType::Literal,
        }
    }
}

impl From<proc_macro::TokenTree> for TokenType {
    fn from(token: proc_macro::TokenTree) -> Self {
        Self::from(&token)
    }
}

impl From<&proc_macro2::TokenTree> for TokenType {
    fn from(token: &proc_macro2::TokenTree) -> Self {
        match token {
            proc_macro2::TokenTree::Group(..) => TokenType::Group,
            proc_macro2::TokenTree::Ident(..) => TokenType::Ident,
            proc_macro2::TokenTree::Punct(..) => TokenType::Punct,
            proc_macro2::TokenTree::Literal(..) => TokenType::Literal,
        }
    }
}

impl From<proc_macro2::TokenTree> for TokenType {
    fn from(token: proc_macro2::TokenTree) -> Self {
        Self::from(&token)
    }
}
