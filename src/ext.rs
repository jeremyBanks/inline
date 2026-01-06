//! Extension traits and utilities for `InlineCell<T>`.
//!
//! This module provides traits with methods that extend `InlineCell<T>` functionality
//! without polluting the namespace of the inner type `T`.

use crate::inline::InlineCell;
use crate::value::Value;
use std::path::Path;

/// Extension methods for `InlineCell<T>` that require explicit import.
///
/// These methods are available on `InlineCell<T>` but only when this trait is in scope.
/// This prevents name collisions with methods on the inner type `T`.
///
/// # Example
///
/// ```no_run
/// use cell::{cell, InlineCellExt};
///
/// let mut x = cell(42);
/// x.value = 100;
/// x.flush()?; // Requires InlineCellExt in scope
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait InlineCellExt<T: Value + 'static> {
    /// Write this code cell's current value to the source file immediately.
    ///
    /// Normally, writes happen on Drop. This method allows explicit control
    /// over when the source file is updated.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source file cannot be read or written
    /// - The cell's position cannot be resolved
    /// - The file has been modified by another process
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cell::{cell, InlineCellExt};
    ///
    /// let mut counter = cell(0);
    /// counter.value = 42;
    /// counter.flush()?; // Write immediately, don't wait for Drop
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Get the source file path for this code cell.
    ///
    /// Returns the path to the Rust source file where this cell is defined.
    fn path(&self) -> &Path;

    /// Get the source line number for this code cell.
    ///
    /// Returns the line number (1-indexed) where this cell appears in the source.
    fn line(&self) -> u32;

    /// Get the source column number for this code cell.
    ///
    /// Returns the column number (0-indexed) where this cell appears in the source.
    fn column(&self) -> u32;

    /// Get the stable index for this code cell, if resolved.
    ///
    /// Returns `Some(index)` if the cell's position has been resolved to a stable
    /// index (Nth call in the file). Returns `None` if the index hasn't been
    /// resolved yet (e.g., for non-existent files in testing scenarios).
    fn index(&self) -> Option<usize>;
}

impl<T: Value + 'static> InlineCellExt<T> for InlineCell<T> {
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
            self.guard.value = self.value.clone();
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
            self.guard.value = self.value.clone();
            self.guard.verify_source(&self.value)?;
            return Ok(());
        }

        // In Write mode, update the source
        if mode.can_write() {
            self.guard.value = self.value.clone();
            self.guard.update_source(&self.value)?;

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
        self.guard.call_index
    }
}

// Re-export as free functions for use without trait import

/// Write a code cell's current value to the source file immediately.
///
/// This is a free function version of [`InlineCellExt::flush`].
///
/// # Example
///
/// ```no_run
/// use cell::cell;
///
/// let mut x = cell(42);
/// x.value = 100;
/// cell::flush(&mut x)?; // No trait import needed
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn flush<T: Value + 'static>(
    cell: &mut InlineCell<T>,
) -> Result<(), Box<dyn std::error::Error>> {
    InlineCellExt::flush(cell)
}

/// Get the source file path for a code cell.
pub fn path<T: Value + 'static>(cell: &InlineCell<T>) -> &Path {
    InlineCellExt::path(cell)
}

/// Get the source line number for a code cell.
pub fn line<T: Value + 'static>(cell: &InlineCell<T>) -> u32 {
    InlineCellExt::line(cell)
}

/// Get the source column number for a code cell.
pub fn column<T: Value + 'static>(cell: &InlineCell<T>) -> u32 {
    InlineCellExt::column(cell)
}

/// Get the stable index for a code cell, if resolved.
pub fn index<T: Value + 'static>(cell: &InlineCell<T>) -> Option<usize> {
    InlineCellExt::index(cell)
}
