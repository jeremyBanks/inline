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

/// A Rust source document (file).
#[derive(Debug, Clone)]
#[doc(alias("file", "tree", "graph", "DOM"))]
pub struct Document {
    inner: Arc<internal::Document>,
}

impl Hash for Document {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.inner).hash(state)
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(Arc::as_ptr(&self.inner), Arc::as_ptr(&other.inner))
    }
}

impl Eq for Document {}

impl Ord for Document {
    fn cmp(&self, other: &Self) -> Ordering {
        Arc::as_ptr(&self.inner).cmp(&Arc::as_ptr(&other.inner))
    }
}

impl PartialOrd for Document {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Document {
    /// Parses Rust source code into an anonymous [`Document`].
    pub fn parse(source: &str) -> Result<Document, ParseError> {
        Ok(Document {
            inner: internal::Document::parse(Arc::new(source.to_string()), None)?,
        })
    }

    /// Parses Rust source code into a named [`Document`].
    pub fn parse_named(source: &str, name: &str) -> Result<Document, ParseError> {
        Ok(Document {
            inner: internal::Document::parse(Arc::new(source.to_string()), Some(name))?,
        })
    }

    /// Returns the full source code of this file as an [`&Arc<str>`][Arc<str>].
    pub fn source(&self) -> &Arc<String> {
        &self.inner.source()
    }

    /// Returns a span covering the full source code of this file,
    /// including any leading or trailing whitespace.
    pub fn span(&self) -> DocumentSpan {
        DocumentSpan {
            document: self.clone(),
            inner: self.inner.span(),
        }
    }

    /// The (file) name, if this [`Document`] has one.
    #[doc(alias("location"))]
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// The root [`Node`] of this [`Document`], a non-delimited [`Group`](NodeType::Group).
    #[doc(alias("documentElement"))]
    pub fn root(&self) -> Node {
        Node {
            document: self.clone(),
            inner: self.inner.root().clone(),
        }
    }

    // pub fn iter(&self) {
    //     self.root().iter()
    // }

    /// Returns a new [`Document`] based on this one by applying a list of
    /// `(original, replacement)` pairs to the source code. Each `original`
    /// must be a distinct non-overlapping [`Node`] in this [`Document`].
    pub fn replace_nodes<'a>(
        &'a self,
        pairs: impl IntoIterator<Item = (&'a Node, &'a Node)>,
    ) -> Document {
        todo!()
    }
}

impl AsRef<str> for Document {
    fn as_ref(&self) -> &str {
        &self.source()
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root().span().as_str())
    }
}
