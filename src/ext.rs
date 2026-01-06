//! Extension traits and utilities for `Literal<T>`.
//!
//! This module provides traits with methods that extend `Literal<T>` functionality
//! without polluting the namespace of the inner type `T`.

use crate::inline::Literal;
use crate::literal::Value;
use std::path::Path;

/// Extension methods for `Literal<T>` that require explicit import.
///
/// These methods are available on `Literal<T>` but only when this trait is in scope.
/// This prevents name collisions with methods on the inner type `T`.
///
/// # Example
///
/// ```no_run
/// use jeb_literal::{literal, LiteralExt};
///
/// let mut x = literal(42);
/// x.literal = 100;
/// x.flush()?; // Requires LiteralExt in scope
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait LiteralExt<T: Value + 'static> {
    /// Write this literal's current value to the source file immediately.
    ///
    /// Normally, writes happen on Drop. This method allows explicit control
    /// over when the source file is updated.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source file cannot be read or written
    /// - The literal's position cannot be resolved
    /// - The file has been modified by another process
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jeb_literal::{literal, LiteralExt};
    ///
    /// let mut counter = literal(0);
    /// counter.literal = 42;
    /// counter.flush()?; // Write immediately, don't wait for Drop
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Get the source file path for this literal.
    ///
    /// Returns the path to the Rust source file where this literal is defined.
    fn path(&self) -> &Path;

    /// Get the source line number for this literal.
    ///
    /// Returns the line number (1-indexed) where this literal appears in the source.
    fn line(&self) -> u32;

    /// Get the source column number for this literal.
    ///
    /// Returns the column number (0-indexed) where this literal appears in the source.
    fn column(&self) -> u32;

    /// Get the stable index for this literal, if resolved.
    ///
    /// Returns `Some(index)` if the literal's position has been resolved to a stable
    /// index (Nth literal in the file). Returns `None` if the index hasn't been
    /// resolved yet (e.g., for non-existent files in testing scenarios).
    fn index(&self) -> Option<usize>;
}

impl<T: Value + 'static> LiteralExt<T> for Literal<T> {
    fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Mark as dirty for tracking
        crate::dirty::mark_dirty(
            &self.guard.file,
            self.guard.line,
            self.guard.column,
        );

        // Get the mode and check if we should write
        let mode = crate::runtime::get_mode();

        // In Memory mode, just update the guard value
        if mode == crate::runtime::Mode::Memory {
            self.guard.value = self.literal.clone();
            return Ok(());
        }

        // In Reject mode, fail
        if mode.should_reject_write() {
            return Err("Write rejected: mode is Reject".into());
        }

        // Resolve index if not already done
        self.guard.resolve_index()?;

        // In Verify mode, verify the value matches source
        if mode == crate::runtime::Mode::Verify {
            self.guard.value = self.literal.clone();
            self.guard.verify_source(&self.literal)?;
            return Ok(());
        }

        // In Write mode, update the source
        if mode.can_write() {
            self.guard.value = self.literal.clone();
            self.guard.update_source(&self.literal)?;

            // Clear dirty flag after successful write
            crate::dirty::clear_dirty(
                &self.guard.file,
                self.guard.line,
                self.guard.column,
            );
        }

        Ok(())
    }

    fn path(&self) -> &Path {
        &self.guard.file
    }

    fn line(&self) -> u32 {
        self.guard.line
    }

    fn column(&self) -> u32 {
        self.guard.column
    }

    fn index(&self) -> Option<usize> {
        self.guard.literal_index
    }
}

// Re-export as free functions for use without trait import

/// Write a literal's current value to the source file immediately.
///
/// This is a free function version of [`LiteralExt::flush`].
///
/// # Example
///
/// ```no_run
/// use jeb_literal::literal;
///
/// let mut x = literal(42);
/// x.literal = 100;
/// jeb_literal::flush(&mut x)?; // No trait import needed
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn flush<T: Value + 'static>(
    literal: &mut Literal<T>,
) -> Result<(), Box<dyn std::error::Error>> {
    LiteralExt::flush(literal)
}

/// Get the source file path for a literal.
pub fn path<T: Value + 'static>(literal: &Literal<T>) -> &Path {
    LiteralExt::path(literal)
}

/// Get the source line number for a literal.
pub fn line<T: Value + 'static>(literal: &Literal<T>) -> u32 {
    LiteralExt::line(literal)
}

/// Get the source column number for a literal.
pub fn column<T: Value + 'static>(literal: &Literal<T>) -> u32 {
    LiteralExt::column(literal)
}

/// Get the stable index for a literal, if resolved.
pub fn index<T: Value + 'static>(literal: &Literal<T>) -> Option<usize> {
    LiteralExt::index(literal)
}
