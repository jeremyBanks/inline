use {
    crate::inner,
    miette::{Diagnostic, SourceCode, SpanContents},
    std::{
        fmt::{Debug, Display},
        ops::Deref,
        sync::Arc,
    },
    thiserror::Error,
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
        expected: NodeType,
        actual: NodeType,
    },
}

/// A Rust source document (file).
#[derive(Debug, Clone)]
#[doc(alias("file", "tree", "graph", "DOM"))]
pub struct Document {
    inner: Arc<inner::Document>,
}

impl Document {
    /// Parses Rust source code into an anonymous [`Document`].
    pub fn parse(source: &str) -> Result<Document, ParseError> {
        Ok(Document {
            inner: inner::Document::parse(source, None)?,
        })
    }

    /// Parses Rust source code into a named [`Document`].
    pub fn parse_named(source: &str, name: &str) -> Result<Document, ParseError> {
        Ok(Document {
            inner: inner::Document::parse(source, Some(name))?,
        })
    }

    /// The (file) name, if this [`Document`] has one.
    #[doc(alias("location"))]
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// The root [`Node`] of this [`Document`], a non-delimited [`Group`](NodeType::Group).
    #[doc(alias("documentElement"))]
    pub fn root(&self) -> Node {
        Node {
            inner: self.inner.root(),
            document: self.clone(),
        }
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root().span().source())
    }
}

/// A node (token) in a Rust source [`Document`].
#[derive(Debug, Clone)]
#[doc(alias("element", "token", "TokenTree"))]
pub struct Node {
    inner: Arc<inner::Node>,
    document: Document,
}
impl Node {
    /// Parses Rust source code into a [`Node`] in an anonymous [`Document`].
    /// If the source contains a single token tree, its `Node` will be returned
    /// directly. Otherwise, a root Group node containing all of the token trees
    /// will be returned.
    pub fn parse(source: &str) -> Result<Node, ParseError> {
        let root = Document::parse(source)?.root();
        Ok(if let [only_child] = &root.children()[..] {
            only_child.clone()
        } else {
            root
        })
    }

    /// What [`NodeType`] variant (corresponding to a [`proc_macro2::TokenTree`]
    /// variant) does this [`Node`] represent?
    pub fn node_type(&self) -> NodeType {
        todo!()
    }

    /// Returns the [`Document`] this [`Node`] is a part of.
    #[doc(alias("ownerDocument"))]
    pub fn document(&self) -> Document {
        self.document.clone()
    }

    /// Returns the [`Node`]s parent, if it has one.
    #[doc(alias("parentNode", "parentElement"))]
    pub fn parent(&self) -> Option<Node> {
        todo!()
    }

    /// Returns the next sibling of the [`Node`], if it has one.
    #[doc(alias("nextSibling", "nextElementSibling"))]
    pub fn next_sibling(&self) -> Option<Node> {
        todo!()
    }

    /// Returns the previous sibling of the [`Node`], if it has one.
    #[doc(alias("previousSibling", "previousElementSibling"))]
    pub fn previous_sibling(&self) -> Option<Node> {
        todo!()
    }

    /// Returns the [`Node`]'s first child, if it has any.
    #[doc(alias("firstChild", "firstElementChild"))]
    pub fn first_child(&self) -> Option<Node> {
        todo!()
    }

    /// Returns the [`Node`]'s last child, if it has any.
    #[doc(alias("lastChild", "lastElementChild"))]
    pub fn last_child(&self) -> Option<Node> {
        todo!()
    }

    /// Returns a [`Vec`] of all of the [`Node`]'s children.
    #[doc(alias("childNodes"))]
    pub fn children(&self) -> Vec<Node> {
        todo!()
    }

    /// Returns a copy of this [`Node`] in its own document.
    #[doc(alias("cloneNode"))]
    pub fn extract(&self) -> Node {
        todo!()
    }

    /// Returns a [`Span`] covering this node.
    /// This won't contain any leading or trailing whitespace, except for the root document node.
    pub fn span(&self) -> Span {
        todo!()
    }

    /// Returns a [`Span`] excluding outer delimiters if this is a delimited group.
    /// For all other nodes this is the same as [`.span()`](Node::span).
    #[doc(alias("innerHTML"))]
    pub fn inner_span(&self) -> Span {
        todo!()
    }

    /// Returns a [`Span`] covering the opening delimiter if this is a delimited group.
    /// For all other nodes this is the same as [`.span()`](Node::span)[`.before()`](Span::before).
    pub fn opening_span(&self) -> Span {
        todo!()
    }

    /// Returns a [`Span`] covering the closing delimiter if this is a delimited group.
    /// For all other nodes this is the same as [`.span()`](Node::span)[`.after()`](Span::after).
    pub fn closing_span(&self) -> Span {
        todo!()
    }

    /// Returns a [`Span`] covering any whitespace (or group delimiters) following this [`Node`].
    /// If there are none, this will be the same as
    /// [`.span()`](Node::span)[`.after()`](Span::after).
    ///
    /// Group delimiters are included because they're they also delimit tokens without being tokens
    /// on their own, i.e. they can determine whether a punctuation is alone or joined.
    pub fn trailing_spacing(&self) -> Span {
        if let Some(next) = self.next() {
            self.span().after().join(next.span().before()).unwrap()
        } else {
            self.span().after()
        }
    }

    /// Returns a [`Span`] covering any whitespace (or group delimiters) preceeding this [`Node`].
    /// If there are none, this will be the same as
    /// [`.span()`](Node::span)[`.before()`](Span::before).
    ///
    /// Group delimiters are included because they're they also delimit tokens without being tokens
    /// on their own, i.e. they can determine whether a punctuation is alone or joined.
    pub fn leading_spacing(&self) -> Span {
        // XXX: this is dumb and bad. .previous() be .parent(), and then where are you?!
        if let Some(previous) = self.previous() {
            self.span().before().join(previous.span().after()).unwrap()
        } else {
            self.span().before()
        }
    }

    /// Returns the next [`Node`] in the [`Document`], if any.
    pub fn next(&self) -> Option<Node> {
        self.first_child()
            .or_else(|| self.next_sibling())
            .or_else(|| self.parent()?.next_sibling())
    }

    /// Returns the previous [`Node`] in the [`Document`], if any.
    pub fn previous(&self) -> Option<Node> {
        self.previous_sibling().or_else(|| self.parent())
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.span().source(), f)
    }
}

/// What type of [`proc_macro2::TokenTree`] a [`Node`] corresponds to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeType {
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

/// A span representing a range index into a Rust source [`Document`].
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[doc(alias("offset"))]
pub struct Span;

impl Span {
    /// The document this span indexes into.
    #[doc(alias("source_file"))]
    pub fn document(&self) -> Document {
        todo!()
    }

    /// Inclusive lower bound of this [`Span`] as a byte index in the [`Document`].
    pub fn lo(&self) -> usize {
        todo!()
    }

    /// Exclusive upper bound of this [`Span`] as a byte index in the [`Document`].
    pub fn hi(&self) -> usize {
        todo!()
    }

    /// Length of this [`Span`] in bytes.
    pub fn len(&self) -> usize {
        self.hi() - self.lo()
    }

    /// Whether this [`Span`] contains any bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a zero-length [`Span`] at the beginning of this [`Span`].
    pub fn before(&self) -> Span {
        todo!()
    }

    /// Returns a zero-length [`Span`] at the end of this [`Span`].
    pub fn after(&self) -> Span {
        todo!()
    }

    /// Returns a new [`Span`] encompassing both `self` and `other`, if they are in the same
    /// [`Document`].
    pub fn join(&self, other: Span) -> Option<Span> {
        todo!()
    }

    /// The span's source code contents as a string.
    pub fn source(&self) -> &str {
        todo!()
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.source(), f)
    }
}

impl AsRef<str> for Span {
    fn as_ref(&self) -> &str {
        self.source()
    }
}
