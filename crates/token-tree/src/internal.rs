//! implementation details

use {
    crate::{
        debug::{debug_once_weak, debug_weak, short_string_debug},
        Location, Span, TokenType,
    },
    core::fmt::Debug,
    once_cell::sync::OnceCell,
    proc_macro2::{self, LexError, LineColumn, TokenStream, TokenTree},
    send_wrapper::SendWrapper,
    std::{
        ops::Deref,
        str::FromStr,
        sync::{Arc, Weak},
    },
};

#[derive(Debug, Clone)]
pub(crate) struct Document {
    pub(crate) name: Option<String>,
    pub(crate) source: Arc<String>,
    pub(crate) root: Arc<Node>,
}

#[derive(Clone)]
pub(crate) struct Node {
    pub(crate) token_type: TokenType,
    pub(crate) document: Weak<Document>,
    pub(crate) span: Span,
    pub(crate) parent: Weak<Node>,
    pub(crate) children: Vec<Arc<Node>>,
    pub(crate) previous_sibling: Weak<Node>,
    pub(crate) next_sibling: OnceCell<Weak<Node>>,
    pub(crate) previous_node: Weak<Node>,
    pub(crate) next_node: OnceCell<Weak<Node>>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&"Node")
            .field("span", &self.span)
            .field("token_type", &self.token_type)
            .field("document", &debug_weak(&self.document))
            .field("parent", &debug_weak(&self.parent))
            .field("previous_node", &debug_weak(&self.previous_node))
            .field("next_node", &debug_once_weak(&self.next_node))
            .field("previous_sibling", &debug_weak(&self.previous_sibling))
            .field("next_sibling", &debug_once_weak(&self.next_sibling))
            .field("children", &self.children)
            .finish()
    }
}
struct ParseState {
    previous_node: Weak<Node>,
    document: Weak<Document>,
    source: Arc<String>,
}

impl Document {
    pub(crate) fn parse(
        source: Arc<String>,
        name: Option<&str>,
    ) -> Result<Arc<Document>, LexError> {
        let token_stream = proc_macro2::TokenStream::from_str(&source)?;
        let name = name.map(ToOwned::to_owned);

        let mut line_offsets = vec![0];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_offsets.push(offset + 1);
            }
        }

        Ok(Arc::new_cyclic(|document| {
            let state = &mut ParseState {
                document: document.clone(),
                source: source.clone(),
                previous_node: Weak::default(),
                // line_offsets,
                ..todo!()
            };

            let root = with_token_children(
                Node {
                    document: document.clone(),
                    parent: Weak::default(),
                    previous_sibling: Weak::default(),
                    next_sibling: OnceCell::default(),
                    previous_node: state.previous_node.clone(),
                    next_node: OnceCell::default(),
                    children: vec![],
                    token_type: TokenType::Group,
                    span: todo!(), /* Span {
                                    * document: document.clone(),
                                    * start: Location {
                                    *     offset: 0,
                                    *     line_column: LineColumn { line: 1, column: 1 },
                                    * },
                                    * end: state.locate_offset(source.len()),
                                    * source: source.clone(),
                                    * pm2: None,
                                    *     ..todo!()
                                    * }, */
                },
                token_stream,
                state,
            );

            fn with_token_children(
                mut parent_inner: Node,
                token_stream: TokenStream,
                state: &mut ParseState,
            ) -> Arc<Node> {
                Arc::new_cyclic(|parent| {
                    let mut previous_sibling = Weak::default();
                    for token in token_stream {
                        let node = Node {
                            document: state.document.clone(),
                            parent: parent.clone(),
                            previous_sibling: previous_sibling.clone(),
                            previous_node: state.previous_node.clone(),
                            next_sibling: OnceCell::default(),
                            next_node: OnceCell::default(),
                            children: vec![],
                            token_type: TokenType::from(&token),
                            span: todo!(), /* Span {
                                            * document: state.document.clone(),
                                            * start: state.locate_line_column(token.span().
                                            * start()),
                                            * end: state.locate_line_column(token.span().end()),
                                            * source: state.source.clone(),
                                            * pm2: Some(SendWrapper::new(token.span())),
                                            *     ..todo!()
                                            * }, */
                        };

                        let node = if let TokenTree::Group(group) = &token {
                            with_token_children(node, group.stream(), state)
                        } else {
                            Arc::new(node)
                        };

                        if let Some(previous_sibling) = previous_sibling.upgrade() {
                            previous_sibling
                                .next_sibling
                                .set(Arc::downgrade(&node))
                                .unwrap();
                        }
                        previous_sibling = Arc::downgrade(&node);

                        // XXX: this is post-order/parents-after-children, but the documentation
                        // says the opposite. what we do we actually want?
                        // maybe we need to nest yet another layer of new_cyclic.
                        if let Some(previous_node) = state.previous_node.upgrade() {
                            previous_node.next_node.set(Arc::downgrade(&node)).unwrap();
                        }
                        state.previous_node = Arc::downgrade(&node);

                        parent_inner.children.push(node);
                    }
                    parent_inner
                })
            };

            Document { root, source, name }
        }))
    }

    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub(crate) fn root(&self) -> &Arc<Node> {
        &self.root
    }

    pub(crate) fn source(&self) -> &Arc<String> {
        &self.source
    }

    pub(crate) fn span(&self) -> Span {
        self.root.span()
    }
}

impl Node {
    pub(crate) fn document(&self) -> Arc<Document> {
        self.document.upgrade().unwrap()
    }

    pub(crate) fn parent(&self) -> Option<Arc<Node>> {
        self.parent.upgrade()
    }

    pub(crate) fn children(&self) -> &[Arc<Node>] {
        &self.children
    }

    pub(crate) fn span(&self) -> Span {
        self.span.clone()
    }
}
