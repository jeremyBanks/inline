use {
    core::fmt::{Debug, Display},
    derive_more::{From, TryInto},
    std::{fmt::Write, sync::Arc},
};

/// A type representing a literal value in the source code that can be edited
/// by `litter`. Using this type with a value that is not actually a literal in
/// your source code may result in logic errors or panics.
pub trait Literal:
    'static
    + Sized
    + Debug
    + PartialEq
    + Clone
    + Copy
    + Default
    + Into<AnyLiteral>
    + TryFrom<AnyLiteral>
{
    /// For strings and byte strings, this is the inner unsized type, to allow
    /// for operations with similarly-typed values with non-static lifetimes. This
    /// is simply `Self` for other types.
    type Inner: 'static + ?Sized + Debug + PartialEq;

    /// Formats this literal as we would like it to appear in source code.
    fn fmt_source(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // By default we use the type's verbose Debug formatting.
        write!(f, "{self:#?}")
    }
}

impl Literal for &'static str {
    type Inner = str;

    fn fmt_source(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut any_quotes_or_backslashes = false;
        let mut octothorpes_following_quote = None;
        let mut max_octothorpes_following_quote = None;

        for char in self.chars() {
            if char == '"' {
                octothorpes_following_quote = Some(0);
                any_quotes_or_backslashes = true;
            } else if char == '#' && let Some(mut current) = octothorpes_following_quote {
                current += 1;
            } else {
                octothorpes_following_quote = None;
                if char == '\\' {
                    any_quotes_or_backslashes = true;
                }
            }
            if let Some(current) = octothorpes_following_quote {
                if let Some(max) = max_octothorpes_following_quote {
                    if current > max {
                        max_octothorpes_following_quote = Some(current);
                    }
                } else {
                    max_octothorpes_following_quote = Some(current);
                }
            }
        }

        let raw = any_quotes_or_backslashes.then_some("r").unwrap_or("");
        let octothorpes = "#".repeat(max_octothorpes_following_quote.map(|n| n + 1).unwrap_or(0));

        write!(f, "{raw}{octothorpes}\"{self}\"{octothorpes}")
    }
}

impl Literal for &'static [u8] {
    type Inner = [u8];

    fn fmt_source(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // XXX: maybe we should use formatter::pad() and similar?
        // XXX: this should probably also use raw strings with octothorpes if neccessary
        let mut s = String::with_capacity((self.len() + 1) * 4);
        let mut last_newline_index = 2;
        let mut previous_was_printable = false;
        write!(s, "b\"").unwrap();
        for byte in self.iter() {
            let line_length = s.len() as i64 - last_newline_index;
            let line_broken = line_length + 4 > 64;
            if line_broken {
                write!(s, "\\\n  ").unwrap();
                last_newline_index = s.len() as i64;
            }
            match byte {
                // a newline following a printable character is converted into an escaped
                // newline to maintain the content and our own indentation, using \n.
                // other newlines are hex-encoded like a non-printable value, using \x10.
                b'\n' => {
                    if previous_was_printable {
                        write!(s, "\\n\\\n  ").unwrap();
                        last_newline_index = s.len() as i64;
                    } else {
                        write!(s, "\\x0A").unwrap();
                    }
                    previous_was_printable = false;
                },

                // leading spaces (following a backslashed-newline) would be trimmed unless encoded
                b' ' => {
                    if line_broken || !previous_was_printable {
                        write!(s, "\\x20").unwrap();
                    } else {
                        write!(s, " ").unwrap();
                    }
                    previous_was_printable = true;
                },

                // most ascii printable characters are left as-is
                b'!' | b'#'..=b'[' | b']'..=b'~' => {
                    write!(s, "{}", *byte as char).unwrap();
                    previous_was_printable = true;
                },

                // but backslash and double quotes are uppercase hex escaped along with everything
                // else
                b'\\' | b'"' | 0..=0x1F | 0x7F..=0xFF => {
                    write!(s, "\\x{byte:02X}").unwrap();
                    previous_was_printable = false;
                },
            }
        }
        s.push('\"');
        write!(f, "{s}")
    }
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

#[allow(non_camel_case_types)]
#[derive(From, TryInto, Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum AnyLiteral {
    string(&'static str),
    bytes(&'static [u8]),
    bool(bool),
    char(char),
    u8(u8),
    u16(u16),
    u32(u32),
    u64(u64),
    u128(u128),
    usize(usize),
    i8(i8),
    i16(i16),
    i32(i32),
    i64(i64),
    i128(i128),
    isize(isize),
    f32(f32),
    f64(f64),
}

impl AnyLiteral {
    pub fn fmt_source(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            AnyLiteral::string(literal) => literal.fmt_source(f),
            AnyLiteral::bytes(literal) => literal.fmt_source(f),
            AnyLiteral::bool(literal) => literal.fmt_source(f),
            AnyLiteral::char(literal) => literal.fmt_source(f),
            AnyLiteral::u8(literal) => literal.fmt_source(f),
            AnyLiteral::u16(literal) => literal.fmt_source(f),
            AnyLiteral::u32(literal) => literal.fmt_source(f),
            AnyLiteral::u64(literal) => literal.fmt_source(f),
            AnyLiteral::u128(literal) => literal.fmt_source(f),
            AnyLiteral::usize(literal) => literal.fmt_source(f),
            AnyLiteral::i8(literal) => literal.fmt_source(f),
            AnyLiteral::i16(literal) => literal.fmt_source(f),
            AnyLiteral::i32(literal) => literal.fmt_source(f),
            AnyLiteral::i64(literal) => literal.fmt_source(f),
            AnyLiteral::i128(literal) => literal.fmt_source(f),
            AnyLiteral::isize(literal) => literal.fmt_source(f),
            AnyLiteral::f32(literal) => literal.fmt_source(f),
            AnyLiteral::f64(literal) => literal.fmt_source(f),
        }
    }
}

impl Display for AnyLiteral {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.fmt_source(f)
    }
}
