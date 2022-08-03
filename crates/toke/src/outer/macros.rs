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

/// Create a [`Node`][crate::Node] from literal Rust syntax.
///
/// This just returns a [`Node`] with the literally provided tokens as-is;
/// it doesn't perform any kind of interpolation or `quote`-like expansion.
///
/// ```
/// let literal = toke::n!(24.5);
/// assert_eq!(literal.node_type(), toke::NodeType::Literal);
/// ```
///
/// ```
/// let literal = toke::n!([1, 2, "three"]);
/// assert_eq!(literal.node_type(), toke::NodeType::Group);
/// assert_eq!(literal.opening_delimiter().as_str(), "[");
/// assert_eq!(literal.closing_delimiter().as_str(), "]");
/// assert_eq!(literal.inner_span().as_str(), "1, 2, \"three\"");
/// ```
#[macro_export]
macro_rules! token {
    ($($token:tt)+) => {
        $crate::Node::parse(
            dbg!(concat!($(stringify!($token), "\n"),+)),
        ).unwrap()
    };
}

/// [`parse`][crate::Node::parse] a [`Node`] from a Rust syntax string or other compatible type.
pub fn token(source: impl Toke) -> Result<Node, ParseError> {
    Node::parse(source.source().as_ref())
}

/// Adapter trait used for types that can be converted to a [`Node`] with [`toke::n()`].
pub trait Toke {
    fn stream(&self) -> Cow<TokenStream> {
        Cow::Owned(TokenStream::from_str(self.source().as_ref()).unwrap())
    }

    fn source(&self) -> Cow<str> {
        self.stream().clone().to_string().into()
    }
}

impl Toke for str {}
impl Toke for String {}
impl Toke for TokenTree {}

use std::{borrow::Cow, str::FromStr};

#[doc(hidden)]
pub use crate::token as n;
