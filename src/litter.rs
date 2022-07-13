use once_cell::sync::OnceCell;
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Clone, Debug)]
pub enum Litter<Literal: crate::Literal> {
    Inline(crate::Inline<Literal>),
    External(crate::External<Literal>),
}

#[derive(Clone, Debug)]
pub struct External<Literal: crate::Literal> {
    loaded: OnceCell<Literal>,
    source: PathBuf,
    target: PathBuf,
}

#[derive(Clone, Debug)]
pub struct Inline<Literal: crate::Literal> {
    value: Literal,
    loaded: OnceCell<Literal>,
    source: PathBuf,
    line: usize,
    column: usize,
}

impl<Literal: crate::Literal> Litter<Literal> {
    #[track_caller]
    pub fn inline(value: Literal) -> Litter<Literal> {
        Litter::Inline(crate::Inline {
            value,
            loaded: OnceCell::new(),
            source: PathBuf::new(),
            line: 0,
            column: 0,
        })
    }

    #[track_caller]
    pub fn external(target: PathBuf) -> Litter<Literal> {
        Litter::External(crate::External {
            loaded: OnceCell::new(),
            source: PathBuf::new(),
            target,
        })
    }

    /// the internal literal value
    pub fn literal(&self) -> &Literal {
        match self {
            Litter::Inline(inline) => &inline.value,
            Litter::External(external) => self.loaded(),
        }
    }

    /// the internal value as loaded from the source file
    pub fn loaded(&self) -> &Literal {
        match self {
            Litter::Inline(inline) => &inline.loaded.get().unwrap(),
            Litter::External(external) => &external.loaded.get().unwrap(),
        }
    }
}

impl<Literal: crate::Literal> Deref for Litter<Literal> {
    type Target = Literal;

    fn deref(&self) -> &Literal {
        self.literal()
    }
}

impl<Literal: crate::Literal> PartialEq<Literal> for Litter<Literal> {
    fn eq(&self, other: &Literal) -> bool {
        if self.literal() == other {
            true
        } else {
            // inequality! record this if we're in replacement mode
            false
        }
    }
}
