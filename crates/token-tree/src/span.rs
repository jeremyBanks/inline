use {
    ::core::iter::once,
    ::{
        core::ops::Range,
        derive_more::{From, Into},
        miette::{SourceOffset, SourceSpan},
        proc_macro2::{
            Delimiter as TokenGroupDelimiter, LineColumn, Spacing as TokenPunctSpacing, TokenStream,
        },
        send_wrapper::SendWrapper,
    },
};

/// A location/offset/index into [`SourceCode`][miette::SourceCode], as represented by
/// [`miette::SourceOffset`] or [`proc_macro2::LineColumn`].
///
/// Values may be one-past-the-end, in terms of bytes of the file,
/// characters in a column, or lines in the file, for use as exclusive
/// end locations of a [`Span`].
///
/// Using a [`Location`] with a [`SourceCode`][miette::SourceCode] other than the one it was
/// created for may produce logic errors or panic unless the files are of identical length with
/// identical line break locations. If you really need to a Location with a different file (do you
/// really? why?), you should convert it through `SourceOffset` or `LineColumn` depending on which
/// type of location you want to preserve in the (likely) event of a conflict.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Location {
    miette: SourceOffset,
    proc_macro2: LineColumn,
}

/// A span/range into [`SourceCode`][miette::SourceCode], as represented by
/// [`miette::SourceSpan`], [`proc_macro2::Span`], or a pair of [`Location`]s.
#[derive(Clone, Debug)]
pub struct Span {
    start: Location,
    end: Location,
    miette: SourceSpan,
    proc_macro2: Option<SendWrapper<proc_macro2::Span>>,
}

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

    /// 1-based line number of this location.
    ///
    /// ## Compatibility
    ///
    /// - 👍 [`core::line!()`], [`core::panic::Location::line()`], [`proc_macro::LineColumn::line`],
    ///   [`proc_macro2::LineColumn::line`], and [`rustc_span::Loc::line`][rustc] are also 1-based.
    /// - ⚠️ [`miette::SpanContents::line()`] is 0-based.
    ///
    /// [rustc]: https://doc.rust-lang.org/stable/nightly-rustc/rustc_span/struct.Loc.html#structfield.line
    pub fn line(&self) -> usize {
        self.proc_macro2.line
    }

    /// 1-based column character number of this location.
    ///
    /// ## Compatibility
    ///
    /// - 👍 [`core::column!()`], [`core::panic::Location::column()`], and
    ///   [`proc_macro::LineColumn::column`] are also 1-based.
    /// - ⚠️ [`miette::SpanContents::column()`], [`proc_macro2::LineColumn::column`], and
    ///   [`rustc_span::Loc::col`][rustc] are 0-based.
    ///
    ///
    /// [rustc]: https://doc.rust-lang.org/stable/nightly-rustc/rustc_span/struct.Loc.html#structfield.col
    pub fn column(&self) -> usize {
        self.proc_macro2.column + 1
    }

    /// Returns a `Location` pointing to the 1-based line and column character numbers in given
    /// source code. These will be clamped to at-most one-past-the-end of the lines, columns, or
    /// bytes.
    pub fn from_line_and_column(source: impl AsRef<str>, line: usize, column: usize) -> Location {
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

    /// As a 0-length [`Span`].
    pub fn into_span(&self) -> Span {
        self.into()
    }

    /// As a [`miette::SourceOffset`].
    pub fn into_miette(&self) -> SourceOffset {
        self.into()
    }

    /// From a [`miette::SourceOffset`] for given source code.
    pub fn from_miette(source: impl AsRef<str>, offset: SourceOffset) -> Location {
        Self::from_offset(source, offset.offset())
    }

    /// From a [`proc_macro2::LineColumn`] for given source code.
    pub fn from_proc_macro2(source: impl AsRef<str>, line_column: LineColumn) -> Location {
        Self::from_line_and_column(source, line_column.line, line_column.column + 1)
    }

    /// As a [`proc_macro2::LineColumn`].
    pub fn into_proc_macro2(&self) -> LineColumn {
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

    pub fn from_locations(start: Location, end: Location) -> Span {
        todo!()
    }

    pub fn from_miette(source: impl AsRef<str>, miette: SourceOffset) -> Span {
        todo!()
    }

    pub fn miette(&self) -> SourceSpan {
        self.into()
    }

    pub fn from_proc_macro2(source: impl AsRef<str>, proc_macro2: proc_macro2::Span) -> Span {
        todo!()
    }

    pub fn from_proc_macro(source: impl AsRef<str>, proc_macro: proc_macro::Span) -> Span {
        Self::from_proc_macro2(source, proc_macro.into())
    }

    pub fn try_into_proc_macro2(&self) -> Option<proc_macro2::Span> {
        self.try_into().ok()
    }

    pub fn try_into_proc_macro(&self) -> Option<proc_macro::Span> {
        self.try_into_proc_macro2().map(proc_macro2::Span::unwrap)
    }
}

impl From<&Location> for Span {
    fn from(location: &Location) -> Span {
        Span::from_locations(*location, *location)
    }
}

impl From<Location> for Span {
    fn from(location: Location) -> Span {
        Span::from_locations(location, location)
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

use ::core::ops::{Bound, RangeBounds};

// impl RangeBounds<Location> for Span {
//     fn start_bound(&self) -> Bound<&Location> {
//         Bound::Included(&self.start())
//     }

//     fn end_bound(&self) -> Bound<&Location> {
//         Bound::Excluded(&self.end())
//     }
// }

// impl RangeBounds<usize> for Span {
//     fn start_bound(&self) -> Bound<&usize> {
//         Bound::Included(&self.start.offset())
//     }

//     fn end_bound(&self) -> Bound<&usize> {
//         Bound::Excluded(&self.end.offset())
//     }
// }
