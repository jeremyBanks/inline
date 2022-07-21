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
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Span {
    source_code: arc::Arc<String>,
    lo: usize,
    hi: usize,
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

impl Document {
    pub fn parse(source: &str) -> Result<Document, ParseError> {
        // let id = NEXT_ANONYMOUS_DOCUMENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Document::parse_named(source, &format!("/tmp/_{id}.rs"))
        todo!()
    }

    pub fn parse_named(source: &str, name: &str) -> Result<Document, ParseError> {
        let source = arc::Strong::new(source.to_string());
        let source_string = source.as_ref();
        let token_stream = proc_macro2::TokenStream::from_str(source.as_ref())?;
        let source = arc::Arc::from(source.clone());
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
    // fn parse_file(source: &str) -> Result<arc::Arc<Node>, ParseError> {
    //     let source = arc::Arc::new(source.to_string());
    //     let token_stream = proc_macro2::TokenStream::from_str(source.as_ref())?;
    //     let mut children = todo!(); // Vec::new();

    //     todo!()
    // }

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

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeType {
    Document { document: arc::Arc<Document> },
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
