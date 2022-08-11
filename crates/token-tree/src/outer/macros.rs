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
        proc_macro2::TokenStream,
        std::sync::Arc,
        thiserror::Error,
    },
};

/// Create a [`Node`][crate::Node] from [Rust `tt` arbitrary token tree syntax](https://doc.rust-lang.org/reference/macros.html).
///
/// This just returns a [`Node`] with the literally provided tokens as-is;
/// it doesn't perform any kind of interpolation or `quote`-like expansion.
///
/// ```
/// let literal = token_tree::n!(24.5);
/// assert_eq!(literal.node_type(), token_tree::NodeType::Literal);
/// ```
///
/// ```
/// let literal = token_tree::n!([1, 2, "three"]);
/// assert_eq!(literal.node_type(), token_tree::NodeType::Group);
/// assert_eq!(literal.opening_delimiter().as_str(), "[");
/// assert_eq!(literal.closing_delimiter().as_str(), "]");
/// assert_eq!(literal.inner_span().as_str(), "1, 2, \"three\"");
/// ```
#[macro_export]
macro_rules! node {
    ($($token:tt)+) => {
        $crate::Node::parse(
            (concat!($(stringify!($token), "\n"),+)),
        ).unwrap()
    };
}

/// Create a [`Document`][crate::Document] from [Rust `tt` arbitrary token tree syntax](https://doc.rust-lang.org/reference/macros.html).
#[macro_export]
macro_rules! document {
    ($($token:tt)+) => {
        $crate::Document::parse(
            (concat!($(stringify!($token), "\n"),+)),
        ).unwrap()
    };
}

/// Create a [`Node`][crate::Node`] from [Rust `item` syntax](https://doc.rust-lang.org/reference/items.html).
#[macro_export]
macro_rules! item {
    ($token:item) => {
        $crate::Node::parse(concat!(stringify!($token), "\n")).unwrap()
    };
}

/// Create a [`Node`][crate::Node`] from [Rust `expr` syntax](https://doc.rust-lang.org/reference/expressions.html).
#[macro_export]
macro_rules! expr {
    ($token:expr) => {
        $crate::Node::parse(concat!(stringify!($token), "\n")).unwrap()
    };
}
