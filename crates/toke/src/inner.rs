use {
    core::{fmt::Debug, hash::Hash, str::FromStr},
    miette::Diagnostic,
    once_cell::sync::OnceCell,
    proc_macro2::{self, LexError},
    std::{
        collections::BTreeMap,
        rc::Weak,
        sync::{atomic::AtomicUsize, Arc},
    },
    thiserror::Error,
};

#[derive(Debug, Clone)]
pub(crate) struct Node {
    document: Weak<Document>,
    parent: Weak<Node>,
    children: Vec<Arc<Node>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Document {
    root: Arc<Node>,
}

pub(crate) struct Span {
    document: Weak<Document>,
}

impl Document {
    pub(crate) fn parse(source: &str, name: Option<&str>) -> Result<Arc<Document>, LexError> {
        todo!()
    }

    pub(crate) fn name(&self) -> Option<&str> {
        todo!()
    }

    pub(crate) fn root(&self) -> Arc<Node> {
        todo!()
    }
}
