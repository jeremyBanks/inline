use {
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

// pub(crate) mod inner;
pub(crate) mod inner {
    struct Node;
    struct Document;
}
mod outer;
pub use self::outer::*;
