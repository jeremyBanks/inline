use {
    crate::*,
    ::{
        core::{
            cmp::Ordering,
            fmt::{Debug, Display},
            hash::{Hash, Hasher},
            iter::FusedIterator,
            ops::Deref,
        },
        miette::{Diagnostic, SourceCode, SpanContents},
        std::sync::Arc,
        thiserror::Error,
    },
};

/// Errors that can arise from [`Document::parse()`] or [`Node::parse()`].
#[derive(Debug, Error, Diagnostic)]
pub enum ParseError {
    #[error("invalid rust token syntax in input")]
    Syntax {
        #[from]
        source: proc_macro2::LexError,
    },
    #[error("wrong type of token: expected {expected:?} but found {actual:?}")]
    Type {
        expected: TokenType,
        actual: TokenType,
    },
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// A line/column position in a [`Document`].
///
/// This is just a simple data type; it's not concretely associated with a specific
/// [`Document`], but using it with a document other than the one is was created from
/// may result in out-of-bounds panics.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineColumn {
    /// a 1-indexed line number in a [`Document`]'s [`Document::source`].
    pub line: usize,
    /// The 0-indexed column (in UTF-8 characters) in a [`Document::source`].
    pub column: usize,
}
