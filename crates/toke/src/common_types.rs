use ::{
    derive_more::{From, Into},
    miette::{SourceOffset, SourceSpan},
    proc_macro2::{
        Delimiter as TokenGroupDelimiter, LineColumn, Spacing as TokenPunctSpacing, TokenStream,
    },
    send_wrapper::SendWrapper,
    std::ops::Range,
};
#[doc(no_inline)]
pub use {miette, proc_macro2};

// We can't actually instantiate anything from `proc_macro`, but we'd like to link it in docs.
extern crate proc_macro;

/// A location/offset/index into [`SourceCode`][miette::SourceCode], as represented by
/// [`miette::SourceOffset`] or [`proc_macro2::LineColumn`].
///
/// Values may be one-past-the-end, in terms of bytes of the file,
/// characters in a column, or lines in the file, for use as exclusive
/// end locations of a [`Span`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Location {
    miette: SourceOffset,
    proc_macro2: LineColumn,
}

/// A span/range into [`SourceCode`][miette::SourceCode], as represented by
/// [`miette::SourceSpan`], [`proc_macro2::Span`], or a pair of [`Location`]s.
#[derive(Clone)]
pub struct Span {
    start: Location,
    end: Location,
    miette: SourceSpan,
    proc_macro2: Option<SendWrapper<proc_macro2::Span>>,
}

/// The variants of [`proc_macro2::TokenTree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenType {
    Group,
    Ident,
    Punct,
    Literal,
}

#[doc(hidden)]
mod impls {
    use {super::*, proc_macro2::TokenTree, std::iter::once};

    impl Location {
        /// 0-based byte offset of this location.
        pub fn offset(&self) -> usize {
            self.miette.offset()
        }

        /// Returns a `Location` pointing to a 0-based byte offset in given source code. These will
        /// be clamped to at-most one-past-the-end of the lines, columns, or bytes. If the offset
        /// doesn't correspond to a valid UTF-8 character index, we'll skip ahead to the next
        /// character (rather than panic).
        pub fn from_offset(source: impl AsRef<str>, offset: usize) -> Location {
            let target_offset = offset;

            let mut line = 1;
            let mut column = 1;
            let mut offset = 0;

            for char in source.as_ref().chars().chain(once('\n')) {
                if target_offset <= offset {
                    break;
                }

                column += 1;
                offset += char.len_utf8();

                if char == '\n' {
                    column = 0;
                    line += 1;
                }
            }

            Location {
                miette: SourceOffset::from(offset),
                proc_macro2: LineColumn {
                    line,
                    column: column - 1,
                },
            }
        }

        pub fn miette(&self) -> SourceOffset {
            self.into()
        }

        /// 1-based line number of this location.
        ///
        /// ## Compatibility
        ///
        /// - 👍 [`core::line!()`], [`core::panic::Location::line()`],
        ///   [`proc_macro::LineColumn::line`], and [`proc_macro2::LineColumn::line`] are also
        ///   1-based.
        /// - ⚠️ [`miette::SpanContents::line()`] is 0-based.
        pub fn line(&self) -> usize {
            self.proc_macro2.line
        }

        /// 1-based column character number of this location.
        ///
        /// ## Compatibility
        ///
        /// - 👍 [`core::column!()`], [`core::panic::Location::column()`], and
        ///   [`proc_macro::LineColumn::column`] are also 1-based.
        /// - ⚠️ [`miette::SpanContents::column()`] and [`proc_macro2::LineColumn::column`] are
        /// 0-based.
        pub fn column(&self) -> usize {
            self.proc_macro2.column + 1
        }

        /// Returns a `Location` pointing to the 1-based line and column character numbers in given
        /// source code. These will be clamped to at-most one-past-the-end of the lines, columns, or
        /// bytes.
        pub fn from_line_column(source: impl AsRef<str>, line: usize, column: usize) -> Location {
            let target_line = line;
            let target_column = column;

            let mut line = 1;
            let mut column = 1;
            let mut offset = 0;

            for char in source.as_ref().chars().chain(once('\n')) {
                if target_line == line && target_column == column {
                    break;
                }

                column += 1;
                offset += char.len_utf8();

                if char == '\n' {
                    if target_line == line {
                        // target column is past end-of-line
                        break;
                    } else {
                        column = 0;
                        line += 1;
                    }
                }
            }

            Location {
                miette: SourceOffset::from(offset),
                proc_macro2: LineColumn {
                    line,
                    column: column - 1,
                },
            }
        }

        pub fn proc_macro2(&self) -> LineColumn {
            self.into()
        }
    }

    impl Span {
        /// Inclusive start [`Location`].
        pub fn start(&self) -> Location {
            self.start
        }

        /// Exclusive end [`Location`].
        pub fn end(&self) -> Location {
            self.end
        }

        pub fn from_locations(source: impl AsRef<str>, start: Location, end: Location) -> Span {
            todo!()
        }

        pub fn from_miette(source: impl AsRef<str>, miette: SourceOffset) -> Span {
            todo!()
        }

        pub fn miette(&self) -> SourceSpan {
            self.into()
        }

        pub fn from_proc_macro2(source: impl AsRef<str>, proc_macro2: LineColumn) -> Span {
            todo!()
        }

        pub fn try_proc_macro2(&self) -> Option<proc_macro2::Span> {
            self.try_into().ok()
        }
    }

    impl From<Location> for usize {
        fn from(location: Location) -> Self {
            location.miette.offset()
        }
    }

    impl From<&Location> for usize {
        fn from(location: &Location) -> Self {
            location.miette.offset()
        }
    }

    impl From<Location> for miette::SourceOffset {
        fn from(location: Location) -> Self {
            location.miette
        }
    }

    impl From<&Location> for miette::SourceOffset {
        fn from(location: &Location) -> Self {
            location.miette
        }
    }

    impl AsRef<miette::SourceOffset> for Location {
        fn as_ref(&self) -> &miette::SourceOffset {
            &self.miette
        }
    }

    impl From<Location> for proc_macro2::LineColumn {
        fn from(location: Location) -> Self {
            location.proc_macro2
        }
    }

    impl From<&Location> for proc_macro2::LineColumn {
        fn from(location: &Location) -> Self {
            location.proc_macro2
        }
    }

    impl AsRef<proc_macro2::LineColumn> for Location {
        fn as_ref(&self) -> &proc_macro2::LineColumn {
            &self.proc_macro2
        }
    }

    impl From<Span> for miette::SourceSpan {
        fn from(span: Span) -> Self {
            span.miette
        }
    }

    impl AsRef<miette::SourceSpan> for Span {
        fn as_ref(&self) -> &miette::SourceSpan {
            &self.miette
        }
    }

    impl From<Span> for Range<usize> {
        fn from(span: Span) -> Self {
            span.start.miette.offset()..span.end.miette.offset()
        }
    }

    impl From<&Span> for Range<usize> {
        fn from(span: &Span) -> Self {
            span.start.miette.offset()..span.end.miette.offset()
        }
    }

    impl From<&Span> for miette::SourceSpan {
        fn from(span: &Span) -> Self {
            span.miette.clone()
        }
    }

    impl TryFrom<Span> for proc_macro2::Span {
        type Error = &'static str;

        fn try_from(span: Span) -> Result<Self, Self::Error> {
            if let Some(span) = span.proc_macro2 {
                if span.valid() {
                    Ok(*span)
                } else {
                    Err("proc_macro2::Span exists, but on a different thread")
                }
            } else {
                Err("proc_macro2::Span does not exist for this Span")
            }
        }
    }

    impl TryFrom<&Span> for proc_macro2::Span {
        type Error = &'static str;

        fn try_from(span: &Span) -> Result<Self, Self::Error> {
            if let Some(span) = &span.proc_macro2 {
                if span.valid() {
                    Ok(proc_macro2::Span::clone(span))
                } else {
                    Err("proc_macro2::Span exists, but on a different thread")
                }
            } else {
                Err("proc_macro2::Span does not exist for this Span")
            }
        }
    }

    impl From<&TokenTree> for TokenType {
        fn from(token: &TokenTree) -> Self {
            match token {
                TokenTree::Group(..) => TokenType::Group,
                TokenTree::Ident(..) => TokenType::Ident,
                TokenTree::Punct(..) => TokenType::Punct,
                TokenTree::Literal(..) => TokenType::Literal,
            }
        }
    }

    impl From<TokenTree> for TokenType {
        fn from(token: TokenTree) -> Self {
            Self::from(&token)
        }
    }
}
