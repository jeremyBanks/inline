#![doc(html_favicon_url = "https://static.jeremy.ca/toke.png")]
#![doc(html_logo_url = "https://static.jeremy.ca/toke.png")]

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
