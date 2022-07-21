pub(crate) use std::sync::{Arc as Strong, Weak};

/// An Arc that may be Strong or Weak.
#[derive(Debug, Clone)]
pub(crate) enum Arc<T> {
    Strong(Strong<T>),
    Weak(Weak<T>),
}

impl<T> Arc<T> {
    /// Returns a new Strong Arc with the given value.
    pub(crate) fn new(t: T) -> Self {
        Self::from_strong(Strong::new(t))
    }

    /// Returns a new Strong Arc with a default value of the inner type.
    pub(crate) fn with_default() -> Self
    where
        T: Default,
    {
        Self::new(T::default())
    }

    /// Returns a new Weak Arc with no value.
    pub(crate) fn empty() -> Self {
        Self::from_weak(Weak::new())
    }

    /// Replaces this Arc with a new Strong Arc with the given value.
    pub(crate) fn replace(&mut self, t: T) {
        *self = Self::new(t)
    }

    /// Clears this Arc, leaving it as a disassociated Weak Arc.
    pub(crate) fn clear(&mut self) {
        *self = Self::empty()
    }

    pub(crate) fn from_strong(strong: Strong<T>) -> Self {
        Arc::Strong(strong)
    }

    pub(crate) fn from_weak(weak: Weak<T>) -> Self {
        Arc::Weak(weak)
    }

    pub(crate) fn strong(&self) -> Option<Strong<T>> {
        match self {
            Arc::Strong(strong) => Some(strong.clone()),
            Arc::Weak(weak) => weak.upgrade(),
        }
    }

    pub(crate) fn weak(&self) -> Weak<T> {
        match self {
            Arc::Strong(strong) => Strong::downgrade(strong),
            Arc::Weak(weak) => weak.clone(),
        }
    }

    /// Upgrades this to a strong Arc in-place.
    pub(crate) fn upgrade(&mut self) -> Option<&mut Self> {
        if let Arc::Weak(weak) = self {
            *self = Arc::Strong(weak.upgrade()?);
        }
        Some(self)
    }

    /// Downgrades to a weak Arc in-place.
    pub(crate) fn downgrade(&mut self) -> &mut Self {
        if let Arc::Strong(strong) = self {
            *self = Arc::Weak(Strong::downgrade(strong));
        }
        self
    }

    /// Upgrades this to a strong Arc in-place. If the value has already been dropped,
    /// replace it with a Default value of the inner type.
    pub(crate) fn upgrade_or_default(&mut self) -> &mut Self
    where
        T: Default,
    {
        if self.upgrade().is_none() {
            self.replace(T::default());
        }
        self
    }

    pub(crate) fn as_ptr(&self) -> *const T {
        match self {
            Arc::Strong(strong) => Strong::as_ptr(strong),
            Arc::Weak(weak) => weak.as_ptr(),
        }
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.as_ptr(), other.as_ptr())
    }

    pub(crate) fn strong_count(&self) -> usize {
        match self {
            Arc::Strong(strong) => Strong::strong_count(strong),
            Arc::Weak(weak) => weak.strong_count(),
        }
    }

    pub(crate) fn weak_count(&self) -> usize {
        match self {
            Arc::Strong(strong) => Strong::weak_count(strong),
            Arc::Weak(weak) => weak.weak_count(),
        }
    }
}

impl<T> From<Strong<T>> for Arc<T> {
    fn from(strong: Strong<T>) -> Self {
        Arc::from_strong(strong)
    }
}

impl<T> From<Weak<T>> for Arc<T> {
    fn from(weak: Weak<T>) -> Self {
        Arc::from_weak(weak)
    }
}

impl<T> From<Arc<T>> for Weak<T> {
    fn from(arc: Arc<T>) -> Self {
        arc.weak()
    }
}

impl<T> Default for Arc<T> {
    fn default() -> Self {
        Arc::empty()
    }
}
