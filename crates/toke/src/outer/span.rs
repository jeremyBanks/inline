use {
    crate::*,
    ::{
        core::{
            cmp::Ordering,
            fmt::{Debug, Display},
            hash::{Hash, Hasher},
            iter::FusedIterator,
            ops::Deref,
        },
        miette::{Diagnostic, SourceCode, SpanContents},
        std::sync::Arc,
        thiserror::Error,
    },
};

/// A span representing a slice into a Rust source [`Document`].
#[derive(Clone)]
#[doc(alias("offset"))]
pub struct Span {
    document: Document,
    inner: inner::Span,
}

impl AsRef<str> for Span {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Span {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.document(), self.start(), self.end()).hash(state)
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        (self.document(), self.start(), self.end()).eq(&(
            other.document(),
            other.start(),
            other.end(),
        ))
    }
}

impl Eq for Span {}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.document(), self.start(), self.end()).cmp(&(
            other.document(),
            other.start(),
            other.end(),
        ))
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Span {
    /// The document this span indexes into.
    #[doc(alias("source_file"))]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Inclusive lower bound of this [`Span`] as a byte index in the [`Document`]'s `.source()`.
    pub fn start(&self) -> usize {
        todo!()
    }

    /// Exclusive upper bound of this [`Span`] as a byte index in the [`Document`]'s `.source()`.
    pub fn end(&self) -> usize {
        todo!()
    }

    /// Returns a zero-length [`Span`] at the beginning of this [`Span`].
    pub fn before(&self) -> Span {
        todo!()
    }

    /// Returns a zero-length [`Span`] at the end of this [`Span`].
    pub fn after(&self) -> Span {
        todo!()
    }

    /// Returns a new [`Span`] encompassing both `self` and `other`, if they are in the same
    /// [`Document`].
    pub fn join(&self, other: Span) -> Option<Span> {
        todo!()
    }

    /// The span's source code contents as a string.
    pub fn as_str(&self) -> &str {
        &self.document.source()[self.start()..self.end()]
    }

    /// The span's source code contents as a string.
    pub fn as_bytes(&self) -> &[u8] {
        &self.document.source().as_bytes()[self.start()..self.end()]
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}
