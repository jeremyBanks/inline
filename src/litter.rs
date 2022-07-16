use crate::LitterHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Litter<Literal: crate::Literal> {
    literal: Literal,
    location: Location,
}

impl<Literal: crate::Literal> Litter<Literal> {
    #[track_caller]
    pub fn new(literal: Literal) -> Self {
        Self {
            literal,
            location: core::panic::Location::caller().into(),
        }
    }

    pub fn edit(&self) -> LitterHandle<Literal> {
        todo!()
    }

    pub fn read(&self) -> Literal {
        // XXX: consider the case where it's updated elsewhere
        // XXX: does this need to check whether we have a handle
        // XXX: for this key already, and if so go to that for
        // XXX: the updated value?
        self.literal
    }

    pub fn write(&self, _value: &Literal::Inner) {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Location {
    file: &'static str,
    line: u32,
    column: u32,
}

impl From<&'static core::panic::Location<'static>> for Location {
    fn from(location: &'static core::panic::Location<'static>) -> Self {
        Location {
            file: location.file(),
            line: location.line(),
            column: location.column(),
        }
    }
}
