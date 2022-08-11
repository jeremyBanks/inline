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
