use crate::{Literal, Litter, LitterHandle};

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
}

impl<Literal: crate::Literal> LiteralExt for Literal {}
