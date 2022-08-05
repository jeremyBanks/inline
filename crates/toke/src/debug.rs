use {
    once_cell::sync::OnceCell,
    std::{fmt::Debug, sync::Weak},
};

#[macro_export(crate)]
macro_rules! fdebug {
    ($($tt:tt)*) => {
        $crate::debug::AsDebug(format!($($tt)*))
    };
}

pub(crate) fn debug_weak<T: Debug>(weak: &Weak<T>) -> crate::debug::AsDebug {
    let t = std::any::type_name::<T>().rsplit("::").next().unwrap();
    if weak.strong_count() > 0 {
        fdebug!("{t} {{ at {:?} }}", weak.as_ptr())
    } else if weak.ptr_eq(&Weak::new()) {
        fdebug!("{t} {{ empty (null weak) }}")
    } else {
        fdebug!("{t} {{ empty (dropped weak) }}")
    }
}

pub(crate) fn debug_once_weak<T: Debug>(weak: &OnceCell<Weak<T>>) -> crate::debug::AsDebug {
    let t = std::any::type_name::<T>().rsplit("::").next().unwrap();
    if let Some(weak) = weak.get() {
        if weak.strong_count() > 0 {
            fdebug!("{t} {{ at {:?} }}", weak.as_ptr())
        } else if weak.ptr_eq(&Weak::new()) {
            fdebug!("{t} {{ empty (null weak) }}")
        } else {
            fdebug!("{t} {{ empty (dropped weak) }}")
        }
    } else {
        fdebug!("{t} {{ empty (not once) }}")
    }
}

pub(crate) struct AsDebug<S: AsRef<str> = String>(S);
impl<S: AsRef<str>> core::fmt::Debug for AsDebug<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0.as_ref())
    }
}
