use {
    std::{fmt, fmt::Debug},
    ::{
        derive_more::{From, TryInto},
        std::{
            borrow::Cow,
            mem::transmute,
            ops::Deref,
            sync::{Arc, Weak},
        },
    },
};

/// A reference that may be weak or strong.
#[derive(Clone, From)]
pub enum Arcane<T> {
    Strong(Arc<T>),
    Weak(Weak<T>),
}

impl<T> Arcane<T> {
    pub fn new(value: T) -> Self {
        Arcane::Strong(Arc::new(value))
    }

    pub fn empty() -> Self {
        Arcane::Weak(Weak::new())
    }

    pub fn as_inner(&self) -> Option<&T> {
        todo!()
    }

    pub fn into_inner(self) -> Option<T> {
        todo!()
    }

    #[inline]
    pub fn weak<'s>(&'s self) -> &'s Weak<T> {
        match self {
            Arcane::Strong(strong) => {
                // XXX: This is currently unsound, see https://stackoverflow.com/q/73314967/1114.
                // XXX: However, we might get a safe alternative before this crate has a stable
                // XXX: release, see https://github.com/rust-lang/rust/pull/100472. For now,
                // XXX: I'm risking this unsound implementation, with some assertions to help
                // XXX: possibly catch any errors. I expect this to work fine in practice.
                let weak = unsafe { transmute::<&'s Arc<T>, &'s Weak<T>>(strong) };
                assert_eq!(weak.as_ptr(), Arc::as_ptr(strong));
                assert_eq!(weak.strong_count(), Arc::strong_count(strong));
                assert_eq!(weak.weak_count(), Arc::weak_count(strong));
                weak
            },
            Arcane::Weak(weak) => &weak,
        }
    }

    #[inline]
    pub fn strong(&self) -> Option<&Arc<T>> {
        match self {
            Arcane::Strong(arc) => Some(arc),
            Arcane::Weak(_) => None,
        }
    }

    #[inline]
    pub fn upgraded(&mut self) -> Option<&Arc<T>> {
        match self {
            Arcane::Strong(strong) => Some(strong),
            Arcane::Weak(weak) => {
                *self = Arcane::Strong(Arc::from(weak.upgrade().unwrap()));
                self.upgraded()
            },
        }
    }

    #[inline]
    pub fn downgraded(&mut self) -> &Weak<T> {
        match self {
            Arcane::Strong(strong) => {
                *self = Arcane::Weak(Arc::downgrade(strong));
                self.weak()
            },
            Arcane::Weak(weak) => weak,
        }
    }

    #[inline]
    fn into_weak(self) -> Weak<T> {
        match self {
            Arcane::Strong(strong) => {
                todo!();
                self.into_weak()
            },
            Arcane::Weak(weak) => weak,
        }
    }

    #[inline]
    fn try_into_strong(self) -> Option<Arc<T>> {
        match self {
            Arcane::Strong(strong) => Some(strong),
            Arcane::Weak(_) => todo!(), //.upgraded().and_then(|_| self.try_into_strong()),
        }
    }
}

impl<T> Deref for Arcane<T> {
    type Target = Weak<T>;

    #[inline]
    fn deref(&self) -> &Weak<T> {
        self.weak()
    }
}

impl<T> AsRef<Weak<T>> for Arcane<T> {
    #[inline]
    fn as_ref(&self) -> &Weak<T> {
        self.weak()
    }
}

impl<T> From<Arcane<T>> for Weak<T> {
    #[inline]
    fn from(other: Arcane<T>) -> Weak<T> {
        other.into_weak()
    }
}

impl<T> TryFrom<Arcane<T>> for Arc<T> {
    type Error = Weak<T>;

    #[inline]
    fn try_from(other: Arcane<T>) -> Result<Arc<T>, Weak<T>> {
        // other
        //     .try_into_strong()
        //     .ok_or_else(todo!(|| other.into_weak()))
        todo!()
    }
}

impl<T> Debug for Arcane<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Arcane::Strong(strong) => strong.fmt(f),
            Arcane::Weak(weak) => weak.fmt(f),
        }
    }
}

impl<T> Default for Arcane<T>
where
    T: Default,
{
    #[inline]
    fn default() -> Arcane<T> {
        Arcane::Strong(Arc::new(T::default()))
    }
}
