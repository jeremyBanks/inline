use {
    crate::{AnyLiteral, Litter},
    derive_more::{From, TryInto},
    std::sync::Arc,
};

#[derive(Debug)]
pub struct LitterHandle<Literal: crate::Literal> {
    pub(crate) original: Litter<Literal>,
    pub(crate) current: Literal,
}

impl<Literal: crate::Literal> LitterHandle<Literal> {
    pub(crate) fn new(litter: Litter<Literal>) -> Arc<Self> {
        Arc::new(Self {
            original: litter,
            current: litter.literal,
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(From, TryInto, Debug)]
#[non_exhaustive]
pub enum AnyLitterHandle {
    string(LitterHandle<&'static str>),
    bytes(LitterHandle<&'static [u8]>),
    bool(LitterHandle<bool>),
    char(LitterHandle<char>),
    u8(LitterHandle<u8>),
    u16(LitterHandle<u16>),
    u32(LitterHandle<u32>),
    u64(LitterHandle<u64>),
    u128(LitterHandle<u128>),
    usize(LitterHandle<usize>),
    i8(LitterHandle<i8>),
    i16(LitterHandle<i16>),
    i32(LitterHandle<i32>),
    i64(LitterHandle<i64>),
    i128(LitterHandle<i128>),
    isize(LitterHandle<isize>),
    f32(LitterHandle<f32>),
    f64(LitterHandle<f64>),
}
