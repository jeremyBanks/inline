use {
    crate::arc,
    core::{
        fmt::Debug,
        hash::Hash,
        ops::{Index, Range},
        str::FromStr,
    },
    miette::Diagnostic,
    once_cell::sync::OnceCell,
    proc_macro2,
    std::{collections::BTreeMap, error::Error, sync::atomic::AtomicUsize},
    thiserror::Error,
};

/// A "span" is a pair of byte indices used to slice into a `.source_code`.
#[derive(Clone)]
pub struct Span {
    source_code: arc::Weak<String>,
    lo: usize,
    hi: usize,
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.source_code
            .as_ptr()
            .cmp(&other.source_code.as_ptr())
            .then_with(|| self.lo.cmp(&other.lo))
            .then_with(|| self.hi.cmp(&other.hi))
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.source_code.as_ptr() == other.source_code.as_ptr()
            && self.lo == other.lo
            && self.hi == other.hi
    }
}

impl Eq for Span {}

impl Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source_code.as_ptr().hash(state);
        self.lo.hash(state);
        self.hi.hash(state);
    }
}

/// A "document", a read-only tree/graph view of a Rust source file.
#[derive(Debug)]
pub struct Document {
    id: usize,
    name: Option<String>,
    source_code: arc::Arc<String>,
    root_node: OnceCell<arc::Arc<Node>>,
    line_offsets: Vec<usize>,
    nodes_by_span: BTreeMap<(usize, usize), arc::Arc<Node>>,
}

/// A "node" in the graph/view of a Rust source file.
#[derive(Debug)]
pub struct Node {
    source: arc::Arc<String>,
    node_type: NodeType,
    span: (usize, usize),
    children: Vec<arc::Arc<Node>>,
    file: arc::Arc<Document>,
    parent: arc::Arc<Node>,
    previous_sibling: arc::Arc<Node>,
    next_sibling: OnceCell<arc::Arc<Node>>,
}

/// An error ocurred while trying to parse or read a source file.
#[derive(Error, Diagnostic, Debug)]
#[diagnostic(url(docsrs))]
pub enum ParseError {
    #[error(transparent)]
    LexError(#[from] proc_macro2::LexError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// Errors that can occur when creating a [[Span]] value.
#[derive(Error, Diagnostic, Debug)]
#[diagnostic(url(docsrs))]
pub enum SpanError {
    #[error("attempted to create span with lo > hi ({lo} > {hi})")]
    NegativeLength { lo: usize, hi: usize },
}

impl Span {
    pub fn as_tuple(&self) -> (usize, usize) {
        (self.0, self.1)
    }

    pub fn as_range(&self) -> Range<usize> {
        self.0..self.1
    }

    pub fn from_range(range: Range<usize>) -> Self {
        Span(range.start, range.end)
    }

    pub fn try_from_tuple(lo: usize, hi: usize) -> Result<Self, SpanError> {
        if lo > hi {
            Err(SpanError::NegativeLength { lo, hi })
        } else {
            Ok(Span(lo, hi))
        }
    }
}
impl Document {
    pub fn parse(source: &str) -> Result<Document, ParseError> {
        // let id = NEXT_ANONYMOUS_DOCUMENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Document::parse_named(source, &format!("/tmp/_{id}.rs"))
        todo!()
    }

    pub fn parse_named(source: &str, name: &str) -> Result<Document, ParseError> {
        let source = arc::Arc::new(source.to_string());
        let token_stream = proc_macro2::TokenStream::from_str(source.as_ref())?;
        let id = {
            static NEXT_ANONYMOUS_DOCUMENT_ID: AtomicUsize = AtomicUsize::new(1);
            NEXT_ANONYMOUS_DOCUMENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        };

        // let mut children = Vec::new();

        let mut line_offsets = vec![];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_offsets.push(offset);
            }
        }

        // let nodes_by_span = Default::default();

        // let file = Ok(arc::Arc::new_cyclic(|root| {
        //     let mut previous = Weakarc::Arc::new();

        //     for token in token_stream.into_iter() {
        //         let node =
        //             TokenNode::parse_token(token, source.clone(), root.clone(),
        // previous.clone());         if let Some(ref previous) = previous.upgrade() {
        //             previous
        //                 .next_sibling
        //                 .set(arc::Arc::downgrade(&previous))
        //                 .unwrap();
        //         }
        //         children.push(node);
        //     }

        //     if let Some(last) = children.last() {
        //         last.next_sibling.set(Default::default());
        //     };

        //     todo!()

        // TokenNode {
        //     node_type: NodeType::File,
        //     span: (0, source.len()),
        //     source,
        //     line_offsets,
        //     children,
        //     parent: arc::Arc::new(),
        //     next_sibling: OnceCell::with_value(Weakarc::Arc::new()),
        //     previous_sibling: arc::Arc::new(),
        // }
        // }));

        // needs more new_cyclic
        // Ok(arc::Arc::new(TokenTree {
        //     nodes_by_span,
        //     line_offsets,
        //     source,
        //     root: OnceCell::new(),
        // }));

        todo!()
    }
}

impl Node {
    fn parse_file(source: &str) -> Result<arc::Arc<Node>, ParseError> {
        let source = arc::Arc::new(source.to_string());
        let token_stream = proc_macro2::TokenStream::from_str(source.as_ref())?;
        let mut children = todo!(); // Vec::new();

        todo!()
    }

    fn parse_token(
        token: proc_macro2::TokenTree,
        file: arc::Arc<String>,
        parent: arc::Arc<Node>,
        previous_sibling: arc::Arc<Node>,
    ) -> arc::Arc<Node> {
        let mut children = todo!(); // Vec::new();
        let node_type = match token {
            proc_macro2::TokenTree::Group(group) => {
                let mut children = Vec::new();
                for token in group.stream().into_iter() {
                    children.push(Node::parse_token(
                        token,
                        file.clone(),
                        parent.clone(),
                        previous_sibling.clone(),
                    ));
                }
                NodeType::Group
            },
            proc_macro2::TokenTree::Punct(punct) => NodeType::Punct,
            proc_macro2::TokenTree::Ident(ident) => NodeType::Ident,
            proc_macro2::TokenTree::Literal(literal) => NodeType::Literal,
        };
        let span = (token.span().start(), token.span().end());
        todo!()
        // let node = arc::Arc::new(TokenNode {
        //     node_type,
        //     span,
        //     file,
        //     parent,
        //     previous_sibling,
        //     next_sibling: OnceCell::with_value(Weakarc::Arc::new()),
        //     children: Vec::new(),
        //     root: parent.clone(),
        // });
        // if let Some(ref previous_sibling) = previous_sibling.upgrade() {
        //     previous_sibling
        //         .next_sibling
        //         .set(arc::Arc::downgrade(&previous_sibling))
        //         .unwrap();
        // }
        // node
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeType {
    File,
    Group,
    Punct,
    Ident,
    Literal,
}

impl Node {
    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }
}

impl AsRef<str> for Node {
    fn as_ref(&self) -> &str {
        &self.source[self.span.0..self.span.1]
    }
}

/// Trivial impls that we don't need to look at very often.
mod impls {
    use super::*;

    impl TryFrom<(usize, usize)> for Span {
        type Error = SpanError;

        fn try_from(value: (usize, usize)) -> Result<Self, Self::Error> {
            Span::try_from_tuple(value.0, value.1)
        }
    }

    impl From<Range<usize>> for Span {
        fn from(range: Range<usize>) -> Self {
            Span::from_range(range)
        }
    }

    impl From<Span> for Range<usize> {
        fn from(span: Span) -> Range<usize> {
            span.as_range()
        }
    }

    impl Debug for Span {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.as_range().fmt(f)
        }
    }
}
