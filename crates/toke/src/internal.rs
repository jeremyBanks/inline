//! implementation details

use {
    crate::{LineColumn, TokenType},
    core::fmt::Debug,
    once_cell::sync::OnceCell,
    proc_macro2::{self, LexError, TokenStream, TokenTree},
    send_wrapper::SendWrapper,
    std::{
        str::FromStr,
        sync::{Arc, Weak},
    },
};

#[derive(Debug, Clone)]
pub(crate) struct Document {
    root: Arc<Node>,
    source: Arc<String>,
    name: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Node {
    document: Weak<Document>,
    parent: Weak<Node>,
    children: Vec<Arc<Node>>,
    previous_sibling: Weak<Node>,
    next_sibling: OnceCell<Weak<Node>>,
    previous_node: Weak<Node>,
    next_node: OnceCell<Weak<Node>>,
    token_type: TokenType,
    span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct Span {
    document: Weak<Document>,
    start: Location,
    end: Location,
    source: Arc<String>,
    /// If this span corresponds directly to a proc_macro2 input span, we hold a wrapped
    /// copy of it here. (This isn't really used yet, but maybe if we want real proc macro
    /// integration later.)
    /// Note that even though they're `!Send`, proc_macro::Span/proc_macro2::Span both
    /// have "trivial drops" (no destructors), so we don't need to worry about ensuring that
    /// they're returned to the original thread for dropping.
    pm2: Option<SendWrapper<proc_macro2::Span>>,
}

/// Simple data type with the byte offset and character line/column of a location in a file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    /// zero-based byte index
    pub offset: usize,
    /// one-based line and column character indices
    pub line_column: LineColumn,
}

/// Internal type used to map between byte indices and line/column character indices
/// and track state during parsing.
///
/// Lots of room for optimizing here, haha.
#[derive(Debug)]
struct ParseState {
    previous_node: Weak<Node>,
    document: Weak<Document>,
    source: Arc<String>,
    line_offsets: Vec<usize>,
}

impl ParseState {
    pub fn locate_offset(&self, offset: usize) -> Location {
        dbg!(&self);

        // special case: one-past-the-end is considered valid
        if offset == self.source.len() {
            return Location {
                offset,
                line_column: LineColumn {
                    line: self.line_offsets.len(),
                    column: &self.source[*self.line_offsets.last().unwrap()..]
                        .chars()
                        .count()
                        + 1,
                },
            };
        }

        let (line_index, line_offset) = self
            .line_offsets
            .iter()
            .enumerate()
            .filter(|(_, line_offset)| **line_offset <= offset)
            .last()
            .unwrap();

        let rest = &self.source[*line_offset..];
        let target_column_offset = offset - line_offset;

        let column_index = if target_column_offset == 0 {
            0
        } else {
            rest.char_indices()
                .enumerate()
                .find(|(_, (offset, _))| *offset == target_column_offset)
                .expect("offset does not align with any character")
                .0
        };

        let line = line_index + 1;
        let column = column_index + 1;

        Location {
            offset,
            line_column: LineColumn { line, column },
        }
    }

    pub fn locate_line_column(&self, line_column: LineColumn) -> Location {
        let LineColumn { line, column } = line_column;
        let offset;

        debug_assert!(line >= 1);
        debug_assert!(column >= 1);

        if line == self.line_offsets.len() + 1 {
            // trailing newline, offset at end of file
            debug_assert_eq!(1, column);
            offset = self.source.len();
        } else {
            let line_offset = self.line_offsets[line - 1];
            let rest = &self.source[line_offset..];
            let column_offset = rest.char_indices().nth(column - 1).unwrap().0;
            offset = line_offset + column_offset;
        }

        Location {
            offset,
            line_column,
        }
    }
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
                line_offsets,
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
                    span: Span {
                        document: document.clone(),
                        start: Location {
                            offset: 0,
                            line_column: LineColumn { line: 1, column: 1 },
                        },
                        end: state.locate_offset(source.len()),
                        source: source.clone(),
                        pm2: None,
                    },
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
                            span: Span {
                                document: state.document.clone(),
                                start: state.locate_line_column(token.span().start()),
                                end: state.locate_line_column(token.span().end()),
                                source: state.source.clone(),
                                pm2: Some(SendWrapper::new(token.span())),
                            },
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
        todo!()
    }
}

impl Span {
    pub(crate) fn document(&self) -> Arc<Document> {
        self.document.upgrade().unwrap()
    }

    pub(crate) fn start(&self) -> Location {
        self.start
    }

    pub(crate) fn end(&self) -> Location {
        self.end
    }

    /// Returns the proc_macro2::Span associated with this span, if
    /// one was provided and we're running on the original thread.
    /// Otherwise returns the (default) `call_site()` span.
    pub(crate) fn pm2(&self) -> proc_macro2::Span {
        if let Some(pm2) = &self.pm2 {
            if pm2.valid() {
                return **pm2;
            }
        }
        return proc_macro2::Span::call_site();
    }
}
