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

/// An [`Iterator`] traversing through a [`Document`]'s [`Node`]s in depth-first pre-order.
#[derive(Debug, Clone, Default)]
pub struct NodeWalker {
    /// next element to be yielded from the iterator, or None if exhausted.
    next: Option<Node>,
    /// optional exclusive element to end iteration at, None if exhausted.
    end: Option<Node>,
}

impl NodeWalker {
    /// Creates a new [`NodeWalker`] starting at the given [`Node`].
    /// If `end` is specified, it must be a [`Node`] occurring after `start` in the same document,
    /// or else this function will panic.
    pub fn new(start: Node, end: Option<Node>) -> NodeWalker {
        if let Some(end) = &end {
            assert_eq!(start.document(), end.document());
            assert!(start.span().start() <= end.span().start());
            if start.span().end() == end.span().start() {
                assert!(start.span().end() > end.span().end());
            }
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

    /// Returns a reference the exclusive end [`Node`] of this iterator, if any.
    pub fn end(&self) -> Option<&Node> {
        self.end.as_ref()
    }
}

impl FusedIterator for NodeWalker {}

impl Document {
    /// Returns an [`Iterator`] walking every [`Node`] in the document (depth-first in-order).
    pub fn walk(&self) -> NodeWalker {
        NodeWalker::new(self.root(), None)
    }
}

impl Node {
    /// Returns an [`Iterator`] walking this [`Node`] and all of its descendants (depth-first
    /// in-order).
    pub fn walk(&self) -> NodeWalker {
        NodeWalker::new(self.clone(), self.next_sibling())
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
