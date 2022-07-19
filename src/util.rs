use std::{
    collections::HashMap, intrinsics::transmute, mem::forget, ops::Deref, rc::Rc, sync::Arc,
};

pub(crate) fn default<T: Default>() -> T {
    T::default()
}

pub(crate) trait Leak {
    type StaticRef: 'static;
    /// Frees this wrapper without running the destructor, returning a 'static
    /// reference to the now never-to-be-dropped target value.
    fn leak(self) -> Self::StaticRef;
}

impl<Inner: 'static + ?Sized> Leak for Box<Inner> {
    type StaticRef = &'static mut Inner;

    fn leak(self) -> Self::StaticRef {
        Box::leak(self)
    }
}

impl<Inner: 'static + ?Sized> Leak for Rc<Inner> {
    type StaticRef = &'static Inner;

    fn leak(self) -> Self::StaticRef {
        let as_ref: &Inner = self.as_ref();
        let as_static_ref: &'static Inner = unsafe { transmute(as_ref) };
        forget(self);
        as_static_ref
    }
}

impl<Inner: 'static + ?Sized> Leak for Arc<Inner> {
    type StaticRef = &'static Inner;

    fn leak(self) -> Self::StaticRef {
        let as_ref: &Inner = self.as_ref();
        let as_static_ref: &'static Inner = unsafe { transmute(as_ref) };
        forget(self);
        as_static_ref
    }
}

impl<Inner: Leak + Clone> Leak for &Inner {
    type StaticRef = <Inner as Leak>::StaticRef;

    fn leak(self) -> Self::StaticRef {
        Inner::clone(self).leak()
    }
}

impl<Inner: 'static> Leak for Vec<Inner> {
    type StaticRef = &'static [Inner];

    fn leak(self) -> Self::StaticRef {
        let as_ref: &[Inner] = self.as_ref();
        let as_static_ref: &'static [Inner] = unsafe { transmute(as_ref) };
        forget(self);
        as_static_ref
    }
}
