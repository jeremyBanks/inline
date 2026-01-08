//! Token preservation wrapper for use with the `tokens!` macro.
//!
//! The `Tokens` type stores arbitrary tokens and implements `Bake` to
//! reproduce them directly, preserving the original token structure.

use databake::{Bake, CrateEnv};
use proc_macro2::TokenStream;

/// A wrapper for arbitrary tokens that preserves them through baking.
///
/// When used with the [`tokens!`](crate::tokens) macro, creates a self-modifying
/// cell that updates its source code when the tokens change.
///
/// # Example
///
/// ```no_run
/// use inline::tokens;
///
/// // Creates an InlineCell<Tokens> that can update its source
/// let mut toks = tokens!(foo bar 123 "hello");
///
/// // Modify the tokens
/// toks.value = inline::Tokens::from_str("new value");
///
/// // On drop, source updates to: tokens!(new value)
/// ```
///
/// For a raw `Tokens` value without the cell wrapper, use [`Tokens::from_str()`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Tokens {
    /// The string representation of the tokens
    source: String,
}

impl Tokens {
    /// Create a new `Tokens` from a `TokenStream`.
    pub fn new(tokens: TokenStream) -> Self {
        Self {
            source: tokens.to_string(),
        }
    }

    /// Create a new `Tokens` from a string.
    ///
    /// The string will be parsed as tokens when needed.
    pub fn from_str(s: &str) -> Self {
        Self {
            source: s.to_string(),
        }
    }

    /// Get the tokens as a `TokenStream`.
    ///
    /// # Panics
    ///
    /// Panics if the stored string is not valid tokens.
    pub fn tokens(&self) -> TokenStream {
        self.source
            .parse()
            .expect("Tokens: stored source should be valid tokens")
    }

    /// Get the raw string representation of the tokens.
    pub fn as_str(&self) -> &str {
        &self.source
    }
}

impl Bake for Tokens {
    fn bake(&self, _ctx: &CrateEnv) -> TokenStream {
        // Output the tokens directly - they get spliced into the macro call site
        // e.g., cell(tokens!(foo bar)) -> the "foo bar" part is what we bake
        self.tokens()
    }
}

impl Default for Tokens {
    fn default() -> Self {
        Self {
            source: String::new(),
        }
    }
}

impl std::fmt::Display for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl From<TokenStream> for Tokens {
    fn from(ts: TokenStream) -> Self {
        Self::new(ts)
    }
}

impl From<&str> for Tokens {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for Tokens {
    fn from(s: String) -> Self {
        Self { source: s }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens_roundtrip() {
        let toks = Tokens::from_str("foo bar 123");
        assert_eq!(toks.as_str(), "foo bar 123");
        assert_eq!(toks.to_string(), "foo bar 123");
    }

    #[test]
    fn test_tokens_bake() {
        let toks = Tokens::from_str("foo bar 123");
        let baked = toks.bake(&CrateEnv::default());
        // Bake outputs raw tokens - they get spliced into the macro call
        assert_eq!(baked.to_string(), "foo bar 123");
    }

    #[test]
    fn test_tokens_equality() {
        let a = Tokens::from_str("x + y");
        let b = Tokens::from_str("x + y");
        let c = Tokens::from_str("x - y");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
