use {
    crate::inner,
    miette::{Diagnostic, SourceCode, SpanContents},
    std::{
        fmt::{Debug, Display},
        hash::Hash,
        ops::Deref,
        sync::Arc,
    },
    thiserror::Error,
};

/// Create a [`Node`] from literal Rust syntax.
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
macro_rules! n {
    ($($token:tt)+) => {
        $crate::Node::parse_named(
            dbg!(concat!($(stringify!($token), "\n"),+)),
            dbg!(concat!(env!("CARGO_MANIFEST_DIR"), "/", file!(), "/toke-n-", line!(), "-", column!(), ".rs")),
        ).unwrap()
    };
}

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

impl Hash for Document {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.inner).hash(state)
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(Arc::as_ptr(&self.inner), Arc::as_ptr(&other.inner))
    }
}

impl Eq for Document {}

impl Ord for Document {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Arc::as_ptr(&self.inner).cmp(&Arc::as_ptr(&other.inner))
    }
}

impl PartialOrd for Document {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
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

    /// Returns the full source code of this file.
    pub fn source(&self) -> &Arc<String> {
        &self.inner.source()
    }

    /// Returns a span covering the full source code of this file,
    /// including any leading or trailing whitespace.
    pub fn span(&self) -> &Span {
        todo!() // self.inner.span()
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
            inner: self.inner.root().clone(),
            document: self.clone(),
        }
    }

    /// Returns a new [`Document`] based on this one by applying a list of
    /// `(original, replacement)` pairs to the source code. Each `original`
    /// must be a distinct non-overlapping [`Node`] in this [`Document`].
    pub fn replace(pairs: impl IntoIterator<Item = (Node, Node)>) -> Document {
        todo!()
    }
}

impl AsRef<Span> for Document {
    fn as_ref(&self) -> &Span {
        self.span()
    }
}

impl Deref for Document {
    type Target = Span;

    fn deref(&self) -> &Self::Target {
        self.span()
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

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span().hash(state)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.span().eq(other.span())
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.span().cmp(other.span())
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
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

    pub fn parse_named(source: &str, name: &str) -> Result<Node, ParseError> {
        let root = Document::parse_named(source, name)?.root();
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
    pub fn document(&self) -> &Document {
        &self.document
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

    /// Returns a [`Span`] covering this node, without any leading or trailing whitespace.
    pub fn span(&self) -> &Span {
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

impl AsRef<Span> for Node {
    fn as_ref(&self) -> &Span {
        self.span()
    }
}

impl Deref for Node {
    type Target = Span;

    fn deref(&self) -> &Self::Target {
        self.span()
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

/// A span representing a slice into a Rust source [`Document`].
#[derive(Clone)]
#[doc(alias("offset"))]
pub struct Span {
    document: Document,
    inner: inner::Span,
}

impl AsRef<str> for Span {
    fn as_ref(&self) -> &str {
        self.source()
    }
}

impl Deref for Span {
    type Target = str;

    fn deref(&self) -> &str {
        self.source()
    }
}

impl Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.document(), self.start(), self.end()).hash(state)
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        (self.document(), self.start(), self.end()).eq(&(
            other.document(),
            other.start(),
            other.end(),
        ))
    }
}

impl Eq for Span {}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.document(), self.start(), self.end()).cmp(&(
            other.document(),
            other.start(),
            other.end(),
        ))
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Span {
    /// The document this span indexes into.
    #[doc(alias("source_file"))]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Inclusive lower bound of this [`Span`] as a byte index in the [`Document`]'s `.source()`.
    pub fn start(&self) -> usize {
        todo!()
    }

    /// Exclusive upper bound of this [`Span`] as a byte index in the [`Document`]'s `.source()`.
    pub fn end(&self) -> usize {
        todo!()
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
        &self.document.inner.source()[self.start()..self.end()]
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.source(), f)
    }
}

/// An [`Iterator`] traversing through a [`Document`]'s [`Node`]s in depth-first pre-order.
pub struct NodeWalker {
    /// next element to be yielded from the iterator, or None if exhausted.
    next: Option<Node>,
    /// optional exclusive element to end iteration at, None if exhausted.
    end: Option<Node>,
}

impl NodeWalker {
    pub fn new(start: Node, end: Option<Node>) -> NodeWalker {
        if let Some(end) = &end {
            debug_assert_eq!(start.document(), end.document());
        }
        NodeWalker {
            next: Some(start),
            end,
        }
    }

    /// Returns a reference to the next Node in this Iterator without advancing it.
    pub fn peek(&self) -> Option<&Node> {
        self.next.as_ref()
    }

    /// Returns the exclusive end [`Node`] of this iterator, if any.
    pub fn end(&self) -> Option<&Node> {
        self.end.as_ref()
    }
}

impl IntoIterator for &Document {
    type IntoIter = NodeWalker;
    type Item = Node;

    /// Iterates over all [`Node`]s in the [`Document`].
    fn into_iter(self) -> Self::IntoIter {
        NodeWalker {
            next: Some(self.root()),
            end: None,
        }
    }
}

impl Document {
    /// Returns an [`Iterator`] every [`Node`] in the document (depth-first in-order).
    pub fn iter(&self) -> NodeWalker {
        self.into_iter()
    }
}

impl IntoIterator for &Node {
    type IntoIter = NodeWalker;
    type Item = Node;

    fn into_iter(self) -> Self::IntoIter {
        NodeWalker {
            end: self.next_sibling(),
            next: Some(self.clone()),
        }
    }
}

impl Node {
    /// Returns an [`Iterator`] this [`Node`] and all of its descendants (depth-first in-order).
    pub fn iter(&self) -> NodeWalker {
        self.into_iter()
    }
}

impl Iterator for NodeWalker {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.next.take()?;
        let next = node.next();
        if next != self.end {
            self.next = next;
        } else {
            self.end = None;
        }
        Some(node)
    }
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
