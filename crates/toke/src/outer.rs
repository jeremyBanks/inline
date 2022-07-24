use {
    crate::inner,
    miette::{Diagnostic, SourceCode, SpanContents},
    std::{fmt::Display, ops::Deref},
    thiserror::Error,
};

#[derive(Debug, Error, Diagnostic)]
pub enum ParseError {
    #[error("invalid rust token syntax in input")]
    Syntax { source: proc_macro2::LexError },
    #[error("wrong type of token: expected {expected:?} but found {actual:?}")]
    Type {
        expected: NodeType,
        actual: NodeType,
    },
}

/// A Rust source document (file).
#[derive(Debug, Clone)]
#[doc(alias("file", "tree", "graph", "DOM"))]
pub struct Document;

impl Document {
    pub fn parse(source: &str) -> Result<Document, ParseError> {
        todo!()
    }

    pub fn parse_named(source: &str, name: &str) -> Result<Document, ParseError> {
        todo!()
    }

    /// The (file) name if this document has one.
    #[doc(alias("location"))]
    pub fn name(&self) -> Option<&str> {
        todo!()
    }

    #[doc(alias("documentElement"))]
    pub fn root(&self) -> Node {
        todo!()
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root().span().source())
    }
}

/// A node (token) in a Rust source [Document].
#[derive(Debug, Clone)]
#[doc(alias("element", "token", "TokenTree"))]
pub struct Node;
impl Node {
    pub fn parse(source: &str) -> Result<Node, ParseError> {
        let root = Document::parse(source)?.root();
        Ok(if let [only_child] = &root.children()[..] {
            only_child.clone()
        } else {
            root
        })
    }

    pub fn node_type(&self) -> NodeType {
        todo!()
    }

    #[doc(alias("ownerDocument"))]
    pub fn document(&self) -> Document {
        todo!()
    }

    #[doc(alias("parentNode", "parentElement"))]
    pub fn parent(&self) -> Option<Node> {
        todo!()
    }

    #[doc(alias("nextSibling", "nextElementSibling"))]
    pub fn next_sibling(&self) -> Option<Node> {
        todo!()
    }

    #[doc(alias("previousSibling", "previousElementSibling"))]
    pub fn previous_sibling(&self) -> Option<Node> {
        todo!()
    }

    #[doc(alias("firstChild", "firstElementChild"))]
    pub fn first_child(&self) -> Option<Node> {
        todo!()
    }

    #[doc(alias("lastChild", "lastElementChild"))]
    pub fn last_child(&self) -> Option<Node> {
        todo!()
    }

    #[doc(alias("childNodes"))]
    pub fn children(&self) -> Vec<Node> {
        todo!()
    }

    #[doc(alias("cloneNode"))]
    pub fn extract(&self) -> Node {
        todo!()
    }

    /// Span covering this node.
    /// This generally won't contain any leading or trailing whitespace, except for
    /// the root document node.
    pub fn span(&self) -> Span {
        todo!()
    }

    /// Span excluding outer delimiters if this is a delimited group.
    /// For all other nodes this is the same as `.span()`.
    #[doc(alias("innerHTML"))]
    pub fn inner_span(&self) -> Span {
        todo!()
    }

    /// Span covering the opening delimiter if this is a delimited group.
    /// For all other nodes this is equivalent to `.span().before()`.
    pub fn opening_span(&self) -> Span {
        todo!()
    }

    /// Span covering the closing delimiter if this is a delimited group.
    /// For all other nodes this is equivalent to `.span().after()`.
    pub fn closing_span(&self) -> Span {
        todo!()
    }

    /// Returns the next Node in the document.
    pub fn next(&self) -> Option<Node> {
        self.first_child()
            .or_else(|| self.next_sibling())
            .or_else(|| self.parent()?.next_sibling())
    }

    /// Returns the previous Node in the document.
    pub fn previous(&self) -> Option<Node> {
        self.previous_sibling().or_else(|| self.parent())
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.span().source())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeType {
    /// A group of tokens, typically wrapped with a delimiter.
    /// The only exception is the root node, which has no delimiter.
    Group,
    /// An identifier.
    Ident,
    /// A literal. (Do we merge in preceding `-` values?)
    /// In most cases it would be correct, but precedence can fuck it up,
    /// and we need to be sure it's actually unary.
    /// `-2.to_string()` is a type error.
    /// So, maybe only in the particular case where they're the only items in
    /// their group? That is unambiguous and covers the 80% case of a single
    /// argument implicitly.
    /// But if we move this into litter, then it can have the luxury of assuming
    /// standard rust grammar -- and split on commas too, giving us 99% of what
    /// we need.
    Literal,
    /// A punctuation. Do we want to merge multiple characters?
    /// Is there any reason not to? Well, in some cases strings
    /// of puncts may be evaluated as separate operators, as in
    /// `dbg!(2----3);`. However... on the token level it would still
    /// be information-preserving and easy to represent that as "----",
    /// and leave any splitting to later.
    /// Maybe we should keep leave this with its sort-of direct mapping
    /// to built-in proc_macro, and leave these heuristics to `litter`.
    Punct,
}

/// A span representing a range index into a Rust source [Document].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[doc(alias("offset"))]
pub struct Span;

impl Span {
    /// The document this span indexes into.
    #[doc(alias("source_file"))]
    pub fn document(&self) -> Document {
        todo!()
    }

    /// Include lower index of the span.
    pub fn lo(&self) -> usize {
        todo!()
    }

    /// Exclusive upper index of the span.
    pub fn hi(&self) -> usize {
        todo!()
    }

    pub fn before(&self) -> Span {
        todo!()
    }

    pub fn after(&self) -> Span {
        todo!()
    }

    pub fn join(&self) -> Option<Span> {
        todo!()
    }

    /// The string contents of the span.
    pub fn source(&self) -> &str {
        todo!()
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source())
    }
}
