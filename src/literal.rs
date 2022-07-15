/// A type representing a literal value in the source code that can be edited
/// by `litter`. Using this type with a value that is not actually a literal in
/// your source code may result in logic errors or panics.
pub trait Literal:
    Sized + 'static + ::core::fmt::Debug + PartialEq + Clone + Copy + Default
{
    /// For strings and byte strings, this is the inner unsized type, to allow
    /// for operations with similarly-typed values with non-static lifetimes. This
    /// is simply `Self` for other types.
    type Inner: ?Sized + 'static + ::core::fmt::Debug + PartialEq;
}

impl Literal for &'static str {
    type Inner = str;
}

impl Literal for &'static [u8] {
    type Inner = [u8];
}

impl Literal for bool {
    type Inner = bool;
}

impl Literal for char {
    type Inner = char;
}

impl Literal for u8 {
    type Inner = u8;
}

impl Literal for u16 {
    type Inner = u16;
}

impl Literal for u32 {
    type Inner = u32;
}

impl Literal for u64 {
    type Inner = u64;
}

impl Literal for u128 {
    type Inner = u128;
}

impl Literal for usize {
    type Inner = usize;
}

impl Literal for i8 {
    type Inner = i8;
}

impl Literal for i16 {
    type Inner = i16;
}

impl Literal for i32 {
    type Inner = i32;
}

impl Literal for i64 {
    type Inner = i64;
}

impl Literal for i128 {
    type Inner = i128;
}

impl Literal for isize {
    type Inner = isize;
}

impl Literal for f32 {
    type Inner = f32;
}

impl Literal for f64 {
    type Inner = f64;
}
