use {
    crate::{Literal, Litter, LitterHandle},
    core::{borrow::Borrow, ops::Index},
};

pub trait LiteralExt: Literal {
    #[track_caller]
    fn litter(&'static self) -> Litter<Self> {
        Litter::new(*self)
    }

    #[track_caller]
    fn edit(&'static self) -> LitterHandle<Self> {
        self.litter().edit()
    }

    #[track_caller]
    fn write(&'static self, value: &Self::Inner) {
        self.litter().write(value)
    }

    #[cfg(feature = "json")]
    #[track_caller]
    fn edit_json(&'static self) -> !
    where
        Self: Index<usize>,
    {
        todo!()
    }

    #[cfg(feature = "postcard")]
    #[track_caller]
    fn edit_postcard(&'static self) -> !
    where
        Self: Borrow<[u8]>,
    {
        todo!()
    }

    #[cfg(feature = "toml")]
    #[track_caller]
    fn edit_toml(&'static self) -> !
    where
        Self: Index<usize>,
    {
        todo!()
    }

    #[cfg(feature = "yaml")]
    #[track_caller]
    fn edit_yaml(&'static self) -> !
    where
        Self: Index<usize>,
    {
        todo!()
    }

    #[cfg(feature = "rkyv")]
    #[track_caller]
    fn edit_rkyv(&'static self) -> !
    where
        Self: Borrow<[u8]>,
    {
        todo!()
    }
}

impl<Literal: crate::Literal> LiteralExt for Literal {}
