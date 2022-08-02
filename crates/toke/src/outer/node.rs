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

/// A node (token) in a Rust source [`Document`].
#[derive(Debug, Clone)]
#[doc(alias("element", "token", "TokenTree"))]
pub struct Node {
    inner: Arc<sync::Node>,
    document: Document,
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
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

    /// Returns a copy of the associated [`Document`] with this Node replaced.
    pub fn replace_with(&self, replacement: Node) -> Document {
        self.document().replace_nodes([(self, &replacement)])
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
        Display::fmt(self.as_str(), f)
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
