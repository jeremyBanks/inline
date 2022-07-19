use derive_more::{From, TryInto};

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

    pub fn edit(&self) -> Litter<Literal> {
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

#[allow(non_camel_case_types)]
#[derive(From, TryInto, Debug)]
#[non_exhaustive]
pub enum AnyLitter {
    string(Litter<&'static str>),
    bytes(Litter<&'static [u8]>),
    bool(Litter<bool>),
    char(Litter<char>),
    u8(Litter<u8>),
    u16(Litter<u16>),
    u32(Litter<u32>),
    u64(Litter<u64>),
    u128(Litter<u128>),
    usize(Litter<usize>),
    i8(Litter<i8>),
    i16(Litter<i16>),
    i32(Litter<i32>),
    i64(Litter<i64>),
    i128(Litter<i128>),
    isize(Litter<isize>),
    f32(Litter<f32>),
    f64(Litter<f64>),
}
