use {
    crate::{Literal, Litter, LitterHandle},
    core::{borrow::Borrow, ops::Index},
};

pub trait LiteralExt: Literal {
    #[track_caller]
    fn litter(&'static self) -> Litter<Self> {
        Litter::new(*self)
    }

    #[cfg(any(doc, all(feature = "write")))]
    #[track_caller]
    fn edit(&'static self) -> LitterHandle<Self> {
        todo!()
    }

    #[cfg(any(doc, all(feature = "write", feature = "json")))]
    #[track_caller]
    fn edit_json(&'static self) -> !
    where
        Self: Index<usize>,
    {
        todo!()
    }

    #[cfg(any(doc, all(feature = "write", feature = "postcard")))]
    #[track_caller]
    fn edit_postcard(&'static self) -> !
    where
        Self: Borrow<[u8]>,
    {
        todo!()
    }

    #[cfg(any(doc, all(feature = "write", feature = "toml")))]
    #[track_caller]
    fn edit_toml(&'static self) -> !
    where
        Self: Index<usize>,
    {
        todo!()
    }

    #[cfg(any(doc, all(feature = "write", feature = "yaml")))]
    #[track_caller]
    fn edit_yaml(&'static self) -> !
    where
        Self: Index<usize>,
    {
        todo!()
    }

    #[cfg(any(doc, all(feature = "write", feature = "rkyv")))]
    #[track_caller]
    fn edit_rkyv(&'static self) -> !
    where
        Self: Borrow<[u8]>,
    {
        todo!()
    }
}

impl<Literal: crate::Literal> LiteralExt for Literal {}
