use {
    core::{
        cmp::Ordering,
        hash::{Hash, Hasher},
    },
    miette::{Diagnostic, IntoDiagnostic},
    std::sync::{Arc, Weak},
    thiserror::Error,
};

#[derive(Debug)]
pub enum Anchor<Inner> {
    Strong(Arc<Inner>),
    Weak(Weak<Inner>),
}

#[derive(Error, Debug, Diagnostic, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("arcane anchor already dropped")]
#[diagnostic(
    code(arcane::DroppedAnchor),
    url(docsrs),
    help("attempted to get a strong reference from a dropped weak reference")
)]
pub struct DroppedAnchor;

impl<Inner> Anchor<Inner> {
    pub fn new(t: Inner) -> Self {
        Self::from_strong(Arc::new(t))
    }

    fn as_ptr(&self) -> *const Inner {
        match self {
            Anchor::Strong(strong) => Arc::as_ptr(strong),
            Anchor::Weak(weak) => weak.as_ptr(),
        }
    }

    pub fn from_strong(strong: Arc<Inner>) -> Self {
        Anchor::Strong(strong)
    }

    pub fn is_strong(&self) -> bool {
        matches!(self, Anchor::Strong(_))
    }

    pub fn strong(&self) -> Option<Arc<Inner>> {
        match self {
            Anchor::Strong(strong) => Some(strong.clone()),
            Anchor::Weak(weak) => weak.upgrade(),
        }
    }

    pub fn upgrade(&mut self) -> Option<&Arc<Inner>> {
        if let Anchor::Weak(weak) = self {
            let strong = weak.upgrade()?;
            debug_assert_eq!(weak.as_ptr(), Arc::as_ptr(&strong));
            *self = Anchor::Strong(strong);
        }
        if let Anchor::Strong(ref strong) = self {
            Some(strong)
        } else {
            unreachable!()
        }
    }

    pub fn from_weak(weak: Weak<Inner>) -> Self {
        Anchor::Weak(weak)
    }

    pub fn is_weak(&self) -> bool {
        matches!(self, Anchor::Weak(_))
    }

    pub fn weak(&self) -> Weak<Inner> {
        match self {
            Anchor::Strong(strong) => Arc::downgrade(strong),
            Anchor::Weak(weak) => weak.clone(),
        }
    }

    pub fn downgrade(&mut self) -> Option<&Weak<Inner>> {
        if let Anchor::Strong(strong) = self {
            let weak = Arc::downgrade(strong);
            debug_assert_eq!(weak.as_ptr(), Arc::as_ptr(strong));
            *self = Anchor::Weak(weak);
        }
        if let Anchor::Weak(ref weak) = self {
            Some(weak)
        } else {
            unreachable!()
        }
    }
}

impl<Inner> From<Arc<Inner>> for Anchor<Inner> {
    fn from(strong: Arc<Inner>) -> Self {
        Anchor::from_strong(strong)
    }
}

impl<Inner> From<Weak<Inner>> for Anchor<Inner> {
    fn from(weak: Weak<Inner>) -> Self {
        Anchor::from_weak(weak)
    }
}

impl<Inner> From<Anchor<Inner>> for Weak<Inner> {
    fn from(arc: Anchor<Inner>) -> Self {
        arc.weak()
    }
}

impl<Inner> TryFrom<Anchor<Inner>> for Arc<Inner> {
    type Error = DroppedAnchor;

    fn try_from(arc: Anchor<Inner>) -> Result<Self, Self::Error> {
        arc.strong().ok_or(DroppedAnchor)
    }
}

impl<Inner> Default for Anchor<Inner>
where
    Inner: Default,
{
    fn default() -> Self {
        Anchor::new(Inner::default())
    }
}

impl<Inner> Ord for Anchor<Inner> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ptr().cmp(&other.as_ptr())
    }
}

impl<Inner> PartialOrd for Anchor<Inner> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ptr().partial_cmp(&other.as_ptr())
    }
}

impl<Inner> PartialEq for Anchor<Inner> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr().eq(&other.as_ptr())
    }
}

impl<Inner> Eq for Anchor<Inner> {}

impl<Inner> Hash for Anchor<Inner> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}

impl<Inner> Clone for Anchor<Inner> {
    fn clone(&self) -> Self {
        match self {
            Anchor::Strong(strong) => Anchor::Strong(strong.clone()),
            Anchor::Weak(weak) => Anchor::Weak(weak.clone()),
        }
    }
}
