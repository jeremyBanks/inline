use {
    arcane::Anchor,
    core::{fmt::Debug, hash::Hash, str::FromStr},
    miette::Diagnostic,
    once_cell::sync::OnceCell,
    proc_macro2,
    std::{
        collections::BTreeMap,
        sync::{atomic::AtomicUsize, Arc},
    },
    thiserror::Error,
};

// Do we need a way to hold multiple strong Arcs in an anchor?
// For example, we want to expose the head of the graph but also want
// to hold on to the tail and be able to deref that.
// Hmm.

/// A "span" is a pair of byte indices used to slice into a `.source_code`.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Span {
    source_code: Anchor<String>,
    lo: usize,
    hi: usize,
}

/// A "document", a read-only tree/graph view of a Rust source file.
#[derive(Debug)]
pub struct Document {
    id: usize,
    name: Option<String>,
    source_code: Anchor<String>,
    root_node: OnceCell<Anchor<Node>>,
    line_offsets: Vec<usize>,
    nodes_by_span: BTreeMap<(usize, usize), Anchor<Node>>,
}

/// A "node" in the graph/view of a Rust source file.
#[derive(Debug)]
pub struct Node {
    source: Anchor<String>,
    node_type: NodeType,
    span: (usize, usize),
    children: Vec<Anchor<Node>>,
    file: Anchor<Document>,
    parent: Anchor<Node>,
    previous_sibling: Anchor<Node>,
    next_sibling: OnceCell<Anchor<Node>>,
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

impl Document {
    pub fn parse(source: &str) -> Result<Document, ParseError> {
        // let id = NEXT_ANONYMOUS_DOCUMENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Document::parse_named(source, &format!("/tmp/_{id}.rs"))
        todo!()
    }

    pub fn parse_named(source: &str, name: &str) -> Result<Document, ParseError> {
        let source = Arc::new(source.to_string());
        let source_string = source.as_ref();
        let token_stream = proc_macro2::TokenStream::from_str(source.as_ref())?;
        let source = Anchor::from(source.clone());
        let id = {
            static NEXT_ANONYMOUS_DOCUMENT_ID: AtomicUsize = AtomicUsize::new(1);
            NEXT_ANONYMOUS_DOCUMENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        };

        // let mut children = Vec::new();

        let mut line_offsets = vec![];
        for (offset, byte) in source_string.bytes().enumerate() {
            if byte == b'\n' {
                line_offsets.push(offset);
            }
        }

        // let nodes_by_span = Default::default();

        // let file = Ok(Anchor::new_cyclic(|root| {
        //     let mut previous = WeakAnchor::new();

        //     for token in token_stream.into_iter() {
        //         let node =
        //             TokenNode::parse_token(token, source.clone(), root.clone(),
        // previous.clone());         if let Some(ref previous) = previous.upgrade() {
        //             previous
        //                 .next_sibling
        //                 .set(Anchor::downgrade(&previous))
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
        //     parent: Anchor::new(),
        //     next_sibling: OnceCell::with_value(WeakAnchor::new()),
        //     previous_sibling: Anchor::new(),
        // }
        // }));

        // needs more new_cyclic
        // Ok(Anchor::new(TokenTree {
        //     nodes_by_span,
        //     line_offsets,
        //     source,
        //     root: OnceCell::new(),
        // }));

        todo!()
    }
}

impl Node {
    // fn parse_file(source: &str) -> Result<Anchor<Node>, ParseError> {
    //     let source = Anchor::new(source.to_string());
    //     let token_stream = proc_macro2::TokenStream::from_str(source.as_ref())?;
    //     let mut children = todo!(); // Vec::new();

    //     todo!()
    // }

    fn parse_token(
        token: proc_macro2::TokenTree,
        file: Anchor<String>,
        parent: Anchor<Node>,
        previous_sibling: Anchor<Node>,
    ) -> Anchor<Node> {
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
                NodeType::Group { delimited: true }
            },
            proc_macro2::TokenTree::Punct(punct) => NodeType::Punct {
                last: punct.spacing() == proc_macro2::Spacing::Alone,
            },
            proc_macro2::TokenTree::Ident(ident) => NodeType::Ident,
            proc_macro2::TokenTree::Literal(literal) => NodeType::Literal,
        };
        let span = (token.span().start(), token.span().end());
        todo!()
        // let node = Anchor::new(TokenNode {
        //     node_type,
        //     span,
        //     file,
        //     parent,
        //     previous_sibling,
        //     next_sibling: OnceCell::with_value(WeakAnchor::new()),
        //     children: Vec::new(),
        //     root: parent.clone(),
        // });
        // if let Some(ref previous_sibling) = previous_sibling.upgrade() {
        //     previous_sibling
        //         .next_sibling
        //         .set(Anchor::downgrade(&previous_sibling))
        //         .unwrap();
        // }
        // node
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeType {
    Group { delimited: bool },
    Punct { last: bool },
    Ident,
    Literal,
}

impl Node {
    pub fn inner_span(&self) -> Span {
        todo!()
    }

    /// Span of this element including any delimiters.
    /// Only relevant for delimited groups.
    pub fn outer_span(&self) -> Span {
        todo!()
    }

    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }
}

impl AsRef<str> for Node {
    fn as_ref(&self) -> &str {
        todo!()
        // &self.source[self.span.0..self.span.1]
    }
}
