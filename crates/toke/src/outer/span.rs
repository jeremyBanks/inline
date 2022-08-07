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

/// A [`Span`] with its [`Document`] attached.
#[derive(Clone)]
#[doc(alias("offset", "range", "index", "region"))]
pub struct DocumentSpan {
    pub(crate) document: Document,
    pub(crate) inner: Span,
}

impl AsRef<str> for DocumentSpan {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for DocumentSpan {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Hash for DocumentSpan {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.document(), self.start(), self.end()).hash(state)
    }
}

impl PartialEq for DocumentSpan {
    fn eq(&self, other: &Self) -> bool {
        (self.document(), self.start(), self.end()).eq(&(
            other.document(),
            other.start(),
            other.end(),
        ))
    }
}

impl Eq for DocumentSpan {}

impl Ord for DocumentSpan {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.document(), self.start(), self.end()).cmp(&(
            other.document(),
            other.start(),
            other.end(),
        ))
    }
}

impl PartialOrd for DocumentSpan {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl DocumentSpan {
    /// The document this span indexes into.
    #[doc(alias("source_file"))]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Returns a zero-length [`Span`] at the beginning of this [`Span`].
    pub fn before(&self) -> DocumentSpan {
        todo!()
    }

    /// Returns a zero-length [`Span`] at the end of this [`Span`].
    pub fn after(&self) -> DocumentSpan {
        todo!()
    }

    /// Returns a new [`Span`] encompassing both `self` and `other`, if they are in the same
    /// [`Document`].
    pub fn join(&self, other: DocumentSpan) -> Option<DocumentSpan> {
        todo!()
    }

    /// The span's source code contents as a string.
    pub fn as_str(&self) -> &str {
        todo!()
    }

    /// The span's source code contents as a string.
    pub fn as_bytes(&self) -> &[u8] {
        todo!()
    }
}

impl Display for DocumentSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl<'this> SpanContents<'this> for &'this DocumentSpan {
    fn data(&self) -> &'this [u8] {
        self.document.source()[core::ops::Range::from(self.inner)].as_bytes()
    }

    fn span(&self) -> &miette::SourceSpan {
        &self.inner.miette()
    }

    fn line(&self) -> usize {
        self.line() - 1
    }

    fn column(&self) -> usize {
        self.column() - 1
    }

    fn line_count(&self) -> usize {
        self.inner.start().line() - self.inner.end().line() + 1
    }
}
