//! _**Pay no attention to that man behind the curtain!**_

use {
    core::fmt::Debug,
    proc_macro2::{self, LexError},
    std::sync::{Arc, Weak},
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
}

#[derive(Debug, Clone)]
pub(crate) struct Span {
    document: Weak<Document>,
    start: usize,
    end: usize,
    source: Arc<String>,
}

impl Document {
    pub(crate) fn parse(source: &str, name: Option<&str>) -> Result<Arc<Document>, LexError> {
        todo!()
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
        todo!()
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

    pub(crate) fn start(&self) -> usize {
        self.start
    }

    pub(crate) fn end(&self) -> usize {
        self.end
    }
}
