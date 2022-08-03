//! 🌳 `toke` provides a read-only graph interface for traversing Rust syntax token trees.
#![doc(html_favicon_url = "https://icon.jeremy.ca/toke.png")]
#![doc(html_logo_url = "https://icon.jeremy.ca/toke.png")]
#![cfg_attr(debug_assertions, allow(unused))]

pub(crate) mod internal;

#[doc(inline)]
pub use self::outer::*;
pub use crate as toke;

#[doc(hidden)]
mod outer;

#[doc(no_inline)]
pub use {
    toke::token as n,
    ::{
        miette::{SourceCode, SourceOffset, SourceSpan},
        proc_macro2::{
            Delimiter as TokenGroupDelimiter, LineColumn, Spacing as TokenPunctSpacing,
            TokenStream, TokenTree,
        },
    },
};

/// The variants of [`proc_macro2::TokenTree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenType {
    /// A group of tokens, typically wrapped with a delimiter (a [`proc_macro2::Group`]).
    ///
    /// The only exception is the root node, which has no delimiter (a bare
    /// [`proc_macro2::TokenStream`]).
    Group,
    /// An identifier (a [`proc_macro2::Ident`]).
    Ident,
    /// A punctuation character (a [`proc_macro2::Punct`]).
    Punct,
    /// A literal (a [`proc_macro2::Literal`]).
    Literal,
}

impl From<&TokenTree> for TokenType {
    fn from(token: &TokenTree) -> Self {
        match token {
            TokenTree::Group(..) => TokenType::Group,
            TokenTree::Ident(..) => TokenType::Ident,
            TokenTree::Punct(..) => TokenType::Punct,
            TokenTree::Literal(..) => TokenType::Literal,
        }
    }
}

impl From<TokenTree> for TokenType {
    fn from(token: TokenTree) -> Self {
        Self::from(&token)
    }
}
