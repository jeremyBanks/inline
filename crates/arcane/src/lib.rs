//! `arcane` is something alright.
#![doc(
    html_favicon_url = "data:image/png;base64,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",
    html_logo_url = "data:image/png;base64,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"
)]
#![cfg_attr(debug_assertions, allow(unused))]

mod anchor;
mod arc;
mod rc;

use {
    core::{marker::PhantomData, mem::transmute, num::NonZeroUsize, ptr::NonNull},
    derive_more::{From, TryInto},
    std::{
        borrow::Cow,
        fmt::{self, Debug},
        ops::Deref,
        rc, sync,
    },
    strum::{AsRefStr, FromRepr},
};

#[derive(Debug, Clone)]
pub struct Arcane<Inner>(EitherArc<Inner>);

#[doc(no_inline)]
pub use Arcane as Arc;

#[derive(Clone)]
pub(crate) enum EitherArc<Inner> {
    Arc(sync::Arc<Inner>),
    Weak(sync::Weak<Inner>),
}

impl<Inner> Debug for EitherArc<Inner>
where
    Inner: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EitherArc::Arc(arc) => arc.fmt(f),
            EitherArc::Weak(weak) => weak.fmt(f),
        }
    }
}

impl<Inner> Arcane<Inner> {
    pub fn new(inner: Inner) -> Self {
        sync::Arc::new(inner).into()
    }

    pub fn from_strong(arc: sync::Arc<Inner>) -> Self {
        arc.into()
    }

    pub fn from_weak(weak: sync::Weak<Inner>) -> Self {
        weak.into()
    }

    pub fn empty() -> Self {
        sync::Weak::new().into()
    }

    pub fn with_default() -> Self
    where
        Inner: Default,
    {
        Arcane::new(Inner::default())
    }

    pub fn make_strong(&mut self) -> &mut Self
    where
        Inner: Default,
    {
        if let EitherArc::Weak(weak) = self.0 {
            if let Some(strong) = weak.upgrade() {
                *self = strong.into();
            } else {
                *self = Inner::default().into();
            }
        }
        self
    }

    pub fn try_make_strong(&mut self) -> Option<&mut Self> {
        if let EitherArc::Weak(weak) = self.0 {
            if let Some(strong) = weak.upgrade() {
                *self = strong.into();
                Some(self)
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    pub fn make_weak(&mut self) -> &mut Self {
        if let EitherArc::Arc(arc) = self.0 {
            *self = self.to_weak().to_owned();
        }
        self
    }

    pub fn make_mut(&mut self) -> &mut Inner
    where
        Inner: Default + Clone,
    {
        match self.0 {
            EitherArc::Arc(ref mut arc) => sync::Arc::make_mut(arc),
            EitherArc::Weak(ref mut weak) => weak.make_mut(),
        }
    }

    pub fn try_make_mut(&mut self) -> Option<&mut Inner>
    where
        Inner: Clone,
    {
        match self.0 {
            EitherArc::Arc(ref mut arc) => Some(sync::Arc::make_mut(arc)),
            EitherArc::Weak(ref mut weak) => None,
        }
    }

    pub fn strong(&self) -> Option<&sync::Arc<Inner>> {
        if let EitherArc::Arc(arc) = &self.0 {
            Some(arc)
        } else {
            None
        }
    }

    pub fn weak(&self) -> Option<&sync::Weak<Inner>> {
        if let EitherArc::Weak(weak) = &self.0 {
            Some(weak)
        } else {
            None
        }
    }

    pub fn is_strong(&self) -> bool {
        matches!(&self.0, EitherArc::Arc(_))
    }

    pub fn is_weak(&self) -> bool {
        matches!(&self.0, EitherArc::Weak(_))
    }

    pub fn to_inner(&self) -> Option<Inner> {
        if let EitherArc::Arc(arc) = &self.0 {
            Some(arc.as_ref())
        } else {
            None
        }
    }

    pub fn unwrap_or_default(&self) -> Inner
    where
        Inner: Default,
    {
        self.to_inner().unwrap_or_else(Inner::default)
    }

    /// Returns a `&Weak` reference to the value.
    pub fn to_weak(&self) -> Cow<sync::Weak<Inner>> {
        match &self.0 {
            EitherArc::Arc(strong) => Cow::Owned(sync::Arc::downgrade(strong)),
            EitherArc::Weak(weak) => Cow::Borrowed(weak),
        }
    }

    /// Returns a `&Strong` reference to the value, unless this reference is dangling.
    pub fn try_to_strong(&self) -> Option<Cow<sync::Arc<Inner>>> {
        match &self.0 {
            EitherArc::Arc(arc) => Some(Cow::Borrowed(arc)),
            EitherArc::Weak(weak) => Some(Cow::Owned(weak.upgrade()?)),
        }
    }

    /// Converts this to a strong reference. If this reference is dangling, initializes a new
    /// backing store containing [`Inner::default()`][Default::default].
    pub fn to_strong(&self) -> Cow<sync::Arc<Inner>>
    where
        Inner: Default,
    {
        match &self.0 {
            EitherArc::Arc(arc) => Cow::Borrowed(arc),
            EitherArc::Weak(weak) => Cow::Owned(weak.upgrade().or_else(|| Inner::default().into())),
        }
    }

    /// ### Safety
    ///
    /// This is not sound under any conditions. It can not be used safely.
    ///
    /// See https://github.com/rust-lang/rust/pull/100472 for the potential safe alternative.
    unsafe fn unsound_as_weak(&self) -> &sync::Weak<Inner> {
        match &self.0 {
            EitherArc::Arc(strong) => {
                let weak = strong as *const sync::Arc<Inner> as *const sync::Weak<Inner>;
                unsafe { &*weak }
            },
            EitherArc::Weak(weak) => weak,
        }
    }

    pub fn into_inner(self) -> Option<Inner> {
        sync::Arc::into_inner(match self.0 {
            EitherArc::Arc(arc) => arc,
            EitherArc::Weak(weak) => weak.upgrade()?,
        })
    }

    pub fn into_weak(self) -> sync::Weak<Inner> {
        match self.0 {
            EitherArc::Arc(arc) => sync::Arc::downgrade(&arc),
            EitherArc::Weak(weak) => weak,
        }
    }

    pub fn try_into_strong(self) -> Option<sync::Arc<Inner>> {
        match self.0 {
            EitherArc::Arc(arc) => Some(arc),
            EitherArc::Weak(weak) => weak.upgrade(),
        }
    }
}

impl<Inner> Default for Arcane<Inner> {
    fn default() -> Self {
        Arcane::empty()
    }
}

impl<Inner> Deref for Arcane<Inner> {
    type Target = sync::Weak<Inner>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.unsound_as_weak() }
    }
}

impl<Inner> AsRef<sync::Weak<Inner>> for Arcane<Inner> {
    fn as_ref(&self) -> &sync::Weak<Inner> {
        self.deref()
    }
}

impl<Inner> From<Inner> for Arcane<Inner> {
    fn from(inner: Inner) -> Self {
        Arcane::new(inner)
    }
}

impl<Inner> From<Option<Inner>> for Arcane<Inner> {
    fn from(inner: Option<Inner>) -> Self {
        match inner {
            Some(inner) => Arcane::new(inner),
            None => Arcane::empty(),
        }
    }
}

impl<Inner> From<sync::Arc<Inner>> for Arcane<Inner> {
    fn from(arc: sync::Arc<Inner>) -> Self {
        Arcane(EitherArc::Arc(arc))
    }
}

impl<Inner> From<sync::Weak<Inner>> for Arcane<Inner> {
    fn from(arc: sync::Weak<Inner>) -> Self {
        Arcane(EitherArc::Weak(arc))
    }
}

impl<Inner> From<Arcane<Inner>> for sync::Weak<Inner> {
    fn from(arc: Arcane<Inner>) -> Self {
        match arc.0 {
            EitherArc::Arc(arc) => sync::Arc::downgrade(&arc),
            EitherArc::Weak(weak) => weak,
        }
    }
}

impl<Inner> TryFrom<Arcane<Inner>> for sync::Arc<Inner> {
    type Error = ();

    fn try_from(arcane: Arcane<Inner>) -> Result<Self, Self::Error> {
        match arcane.0 {
            EitherArc::Arc(arc) => Ok(arc),
            EitherArc::Weak(weak) =>
                if let Some(arc) = weak.upgrade() {
                    Ok(arc)
                } else {
                    Err(())
                },
        }
    }
}
