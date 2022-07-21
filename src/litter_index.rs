use {
    crate::{litter::Location, litter_handle::AnyLitterHandle},
    once_cell::sync::Lazy,
    parking_lot::RwLock,
    std::{
        collections::{BTreeMap, HashMap},
        sync::{Arc, Weak},
    },
};

pub(crate) static INDEX: Lazy<RwLock<HashMap<Location, Arc<AnyLitterHandle>>>> =
    Lazy::new(Default::default);
