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

/// Create a [`Document`][crate::Document] from [Rust `tt` arbitrary token tree syntax](https://doc.rust-lang.org/reference/macros.html).
#[macro_export]
macro_rules! document {
    ($($token:tt)+) => {
        $crate::Document::parse(
            dbg!(concat!($(stringify!($token), "\n"),+)),
        ).unwrap()
    };
}

/// Create a [`Document`][crate::Document] from any number of [Rust `item`s](https://doc.rust-lang.org/reference/items.html), as you might find at the root of [a module](https://doc.rust-lang.org/reference/items/modules.html).
///
/// Top-level inner attributes are supported due to macro grammar limitations.
#[macro_export]
macro_rules! rs {
    (
        $($item:item)*
    ) => {
        $crate::Document::parse(
            dbg!(concat!($(stringify!($item), "\n"),+)),
        ).unwrap()
    };
}

/// Create a [`Node`][crate::Node] from Rust [inner attributes syntax](https://doc.rust-lang.org/reference/attributes.html).
#[doc(alias("meta", "attr"))]
#[macro_export]
macro_rules! inner_attributes {
    (
        $(#![$meta:meta])+
    ) => {
        $crate::Node::parse(
            dbg!(concat!($(stringify!(#![$meta]), "\n"),+)),
        ).unwrap()
    };
}

/// Create a [`Node`][crate::Node] from Rust [outer attributes syntax](https://doc.rust-lang.org/reference/attributes.html).
#[doc(alias("meta", "attr"))]
#[macro_export]
macro_rules! attributes {
    (
        $(#[$meta:meta])+
    ) => {
        $crate::Node::parse(
            dbg!(concat!($(stringify!(#[$meta]), "\n"),+)),
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

/// Create a [`Node`][crate::Node`] from [Rust `stmt` syntax](https://doc.rust-lang.org/reference/statements.html).
#[macro_export]
#[doc(alias("stmt"))]
macro_rules! statement {
    ($token:stmt) => {
        $crate::Node::parse(concat!(stringify!($token), "\n")).unwrap()
    };
}

/// Create a [`Node`][crate::Node`] from [Rust `block` syntax](https://doc.rust-lang.org/reference/block-expr.html).
#[macro_export]
macro_rules! block {
    ($token:block) => {
        $crate::Node::parse(concat!(stringify!($token), "\n")).unwrap()
    };
}

/// Create a [`Node`][crate::Node`] from [Rust `ty` syntax](https://doc.rust-lang.org/reference/types.html).
#[doc(alias("type", "ty"))]
#[macro_export]
macro_rules! tipe {
    ($token:ty) => {
        $crate::Node::parse(concat!(stringify!($token), "\n")).unwrap()
    };
}

/// Create a [`Node`][crate::Node`] from [Rust `path` syntax](https://doc.rust-lang.org/reference/paths.html).
#[macro_export]
macro_rules! path {
    ($token:path) => {
        $crate::Node::parse(concat!(stringify!($token), "\n")).unwrap()
    };
}

/// Create a [`Node`][crate::Node`] from [Rust `expr` syntax](https://doc.rust-lang.org/reference/expressions.html).
#[macro_export]
#[doc(alias("expr"))]
macro_rules! expression {
    ($token:expr) => {
        $crate::Node::parse(concat!(stringify!($token), "\n")).unwrap()
    };
}

/// Create a [`Node`][crate::Node`] from [Rust `ident` syntax](https://doc.rust-lang.org/reference/identifiers.html).
#[macro_export]
#[doc(alias("ident"))]
macro_rules! identifier {
    ($token:ident) => {
        $crate::Node::parse(concat!(stringify!($token), "\n")).unwrap()
    };
}

/// Create a [`Node`][crate::Node`] from [Rust `literal` syntax](https://doc.rust-lang.org/reference/expressions/literal-expr.html).
#[macro_export]
macro_rules! literal {
    ($token:literal) => {
        $crate::Node::parse(concat!(stringify!($token), "\n")).unwrap()
    };
}
