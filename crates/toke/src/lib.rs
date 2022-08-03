//! 🌳 `toke` provides a read-only graph interface for traversing Rust syntax token trees.
#![doc(html_favicon_url = "https://icon.jeremy.ca/toke.png")]
#![doc(html_logo_url = "https://icon.jeremy.ca/toke.png")]
#![cfg_attr(debug_assertions, allow(unused))]

pub(crate) mod internal;

#[doc(inline)]
pub use self::outer::*;

#[doc(hidden)]
mod outer;

/// <code>= [miette::SourceCode];</code>
pub use miette::SourceCode;
/// <code>= [miette::SourceOffset];</code>
pub use miette::SourceOffset;
/// <code>= [miette::SourceSpan];</code>
pub use miette::SourceSpan;
/// <code>= [proc_macro2::Delimiter];</code>
pub use proc_macro2::Delimiter as GroupDelimiter;
/// <code>= [proc_macro2::LineColumn];</code>
pub use proc_macro2::LineColumn;
/// <code>= [proc_macro2::Spacing];</code>
pub use proc_macro2::Spacing as PunctuationSpacing;
/// <code>= [proc_macro2::TokenStream];</code>
pub use proc_macro2::TokenStream;

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
