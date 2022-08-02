//! 🌳 `toke` provides a read-only graph interface for traversing Rust syntax token trees.
#![doc(html_favicon_url = "https://icon.jeremy.ca/toke.png")]
#![doc(html_logo_url = "https://icon.jeremy.ca/toke.png")]
#![cfg_attr(debug_assertions, allow(unused))]

pub(crate) mod inner;

#[doc(inline)]
pub use self::outer::*;

#[doc(hidden)]
mod outer;

/// is [`proc_macro2::LineColumn`]
pub type LineColumn = proc_macro2::LineColumn;
/// is [`proc_macro2::Delimiter`]
pub type GroupDelimiter = proc_macro2::Delimiter;
/// is [`proc_macro2::Spacing`]
pub type PunctuationSpacing = proc_macro2::Spacing;

/// The different types of [`proc_macro2::TokenTree`], as seen by our [`Node`]s.
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

impl TokenType {
    pub fn of(token: &proc_macro2::TokenTree) -> TokenType {
        match token {
            proc_macro2::TokenTree::Group(..) => TokenType::Group,
            proc_macro2::TokenTree::Ident(..) => TokenType::Ident,
            proc_macro2::TokenTree::Punct(..) => TokenType::Punct,
            proc_macro2::TokenTree::Literal(..) => TokenType::Literal,
        }
    }
}
