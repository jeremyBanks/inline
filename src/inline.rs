use crate::literal::Value;
use std::ops::Deref;
use std::path::PathBuf;

/// Internal implementation of a self-modifying value.
///
/// This type wraps a value and provides the ability to update both the
/// in-memory value and its representation in the source code file.
///
/// **Note:** This is an internal type. Users should interact with the [`Literal`]
/// wrapper returned by the `literal!` macro instead.
#[doc(hidden)]
pub struct LiteralInner<T: Value> {
    pub(crate) value: T,
    pub(crate) file: PathBuf,
    pub(crate) line: u32,
    pub(crate) column: u32,
    /// Stable index into the file's literal macros (resolved lazily)
    /// This never changes even if line numbers shift!
    pub(crate) macro_index: Option<usize>,
}

impl<T: Value> LiteralInner<T> {
    /// Create a new LiteralInner instance (called by the registry)
    /// Does NOT fail if the source file doesn't exist - that's only an error if you call set()
    pub(crate) fn new(value: T, file: &str, line: u32, column: u32) -> Self {
        LiteralInner {
            value,
            file: PathBuf::from(file),
            line,
            column,
            macro_index: None, // Resolve lazily when needed
        }
    }

    /// Lazily resolve the macro index from the source file
    pub(crate) fn resolve_index(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        if let Some(index) = self.macro_index {
            return Ok(index);
        }

        let index =
            crate::runtime::get_macro_index(&self.file, self.line, self.column).map_err(|e| {
                format!(
                    "Failed to find literal! macro at {}:{}:{}\n{}",
                    self.file.display(),
                    self.line,
                    self.column,
                    e
                )
            })?;

        self.macro_index = Some(index);
        Ok(index)
    }

    /// Get a reference to the current value
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Update the value and possibly persist to source file (depending on mode)
    ///
    /// Behavior depends on current mode:
    /// - Memory: Just updates the in-memory value (no file I/O)
    /// - Verify: Checks that new value matches what's in the source file, panics if not
    /// - Write: Writes the new value back to the source file
    /// - Reject: Always panics when trying to write
    pub fn set(&mut self, new_value: T) {
        // Compare using PartialEq to detect changes
        if self.value == new_value {
            return; // No change needed
        }

        let mode = crate::runtime::get_mode();

        // In Memory mode, just change the value in memory (no file I/O)
        if mode == crate::runtime::Mode::Memory {
            self.value = new_value; // Move directly, no clone needed
            return;
        }

        // In Reject mode, fail immediately
        if mode.should_reject_write() {
            panic!(
                "Attempted to write in Reject mode at {}:{}:{}",
                self.file.display(),
                self.line,
                self.column
            );
        }

        // For Verify or Write modes, we need file access
        // Resolve the index (lazily loads the file)
        // This is where we'll fail if the file doesn't exist or position is invalid
        if let Err(e) = self.resolve_index() {
            panic!("Failed to access source file: {}", e);
        }

        // In Verify mode: check that the new value matches the source file
        if mode == crate::runtime::Mode::Verify {
            if let Err(e) = self.verify_source(&new_value) {
                panic!(
                    "Literal verification failed at {}:{}:{}\n{}",
                    self.file.display(),
                    self.line,
                    self.column,
                    e
                );
            }
            self.value = new_value; // Move after verification succeeds
            return;
        }

        // In Write mode: write changes to disk
        if mode.can_write() {
            // Check if we're running under cargo
            if !is_running_under_cargo() {
                panic!(
                    "Cannot write to source files outside of cargo environment!\n\
                     File: {}:{}:{}\n\
                     Hint: Run with 'cargo run' or 'cargo test', or use LITERAL_MODE=memory",
                    self.file.display(),
                    self.line,
                    self.column
                );
            }

            if let Err(e) = self.update_source(&new_value) {
                panic!("Failed to write to source file: {}", e);
            }

            // Only update in-memory value after successful write
            self.value = new_value;
        }
    }

    /// Internal: verify that the new value matches what's in the source file
    pub(crate) fn verify_source(&self, new_value: &T) -> Result<(), Box<dyn std::error::Error>> {
        // Index must be resolved by now
        let index = self
            .macro_index
            .expect("Index should be resolved before calling verify_source");

        // Get the current tokens from the source file
        let current_tokens = crate::runtime::get_macro_tokens_by_index(&self.file, index)?;

        // Bake the new value to Rust code
        let env = databake::CrateEnv::default();
        let expected_tokens = new_value.bake(&env);

        // Parse both token streams and compare the parsed AST instead of string representation
        // This handles formatting differences like trailing commas and module paths
        let current_expr: syn::Expr = syn::parse2(current_tokens.clone()).map_err(|e| {
            format!("Failed to parse source tokens: {}", e)
        })?;

        let expected_expr: syn::Expr = syn::parse2(expected_tokens.clone()).map_err(|e| {
            format!("Failed to parse baked tokens: {}", e)
        })?;

        // Compare AST semantically using syn's PartialEq implementation
        // This handles formatting differences like trailing commas, whitespace, and module paths
        if current_expr != expected_expr {
            return Err(format!(
                "Value mismatch!\n  Expected: {}\n  Found in source: {}",
                expected_tokens, current_tokens
            )
            .into());
        }

        Ok(())
    }

    /// Internal: update the source file with the new value
    pub(crate) fn update_source(&self, new_value: &T) -> Result<(), Box<dyn std::error::Error>> {
        // Index must be resolved by now
        let index = self
            .macro_index
            .expect("Index should be resolved before calling update_source");

        // Bake the value to Rust code
        let env = databake::CrateEnv::default();
        let baked_tokens = new_value.bake(&env);

        // Update the shared in-memory state
        crate::runtime::update_macro_by_index(&self.file, index, baked_tokens)?;

        // Write to disk (character-range splicing preserves original formatting)
        crate::runtime::write_to_disk(&self.file)?;

        Ok(())
    }
}

impl<T: Value> Deref for LiteralInner<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Value + std::fmt::Debug> std::fmt::Debug for LiteralInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiteralInner")
            .field("value", &self.value)
            .field("file", &self.file)
            .field("line", &self.line)
            .field("column", &self.column)
            .field("macro_index", &self.macro_index)
            .finish()
    }
}

/// Check if we're running under cargo by looking for cargo-specific env vars
fn is_running_under_cargo() -> bool {
    crate::runtime::is_running_under_cargo()
}

/// A self-modifying value that holds a lock and can update its source code.
///
/// This type wraps a `MutexGuard` to an [`LiteralInner<T>`] and provides
/// convenient access to the value with a single dereference.
///
/// Created via the [`literal!`](macro@crate::literal) macro. The lock is held
/// for the entire lifetime of this value.
///
/// # Example (write-on-drop with .value field)
///
/// ```no_run
/// use jeb_literal::literal;
///
/// let mut counter = literal!(0u32);
/// println!("Value: {}", *counter);  // Single deref
/// counter.literal = *counter + 1;
/// ```
///
/// # Example (write-on-drop with DerefMut)
///
/// ```no_run
/// use jeb_literal::literal;
///
/// let mut counter = literal!(0u32);
/// *counter += 1;  // Mutate directly
/// // Value is automatically written on drop
/// ```
///
/// # Example (public .value field)
///
/// ```no_run
/// use jeb_literal::literal;
///
/// let mut counter = literal!(0u32);
/// counter.literal = 42;  // Direct field assignment, no * needed
/// // Value is automatically written on drop
/// ```

/// Private trait for internal methods that shouldn't pollute the namespace.
///
/// This trait contains methods that are only meant to be called by the macro
/// or internal library code. By making them trait methods, we completely avoid
/// name collisions even with `__` prefixed names.
#[doc(hidden)]
pub trait LiteralPrivate<T: Value + 'static> {
    /// Create a new Literal value (internal use only, called by macro).
    ///
    /// **Note:** For testing only. Leaks the file path string.
    fn __new(value: T, file: &str, line: u32, column: u32) -> Self;
}

pub struct Literal<T: Value + 'static> {
    /// The current literal value. Mutating this field triggers write-on-drop.
    pub literal: T,
    pub(crate) guard: parking_lot::MutexGuard<'static, LiteralInner<T>>,
    /// Clone of the original value when this Literal was created.
    /// Used in Drop to detect mutations.
    original: T,
}

impl<T: Value + 'static> Literal<T> {
    /// Create an Literal wrapper from a mutex guard
    #[doc(hidden)]
    pub fn from_guard(guard: parking_lot::MutexGuard<'static, LiteralInner<T>>) -> Self {
        // Clone the value twice: once for working copy, once for change detection
        let literal = guard.value.clone();
        let original = guard.value.clone();
        Literal { literal, guard, original }
    }

    /// Get a reference to the current value.
    ///
    /// This is equivalent to using the `Deref` implementation (`&*literal`).
    /// The `Deref` implementation is generally preferred for more idiomatic code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jeb_literal::literal;
    ///
    /// let counter = literal!(42u32);
    ///
    /// // These are all equivalent:
    /// let x = counter.get();  // Explicit method call
    /// let x = &*counter;      // Deref (most common)
    /// let x = counter.as_ref(); // AsRef trait
    /// ```
    pub fn get(&self) -> &T {
        &self.literal
    }
}

impl<T: Value + 'static> LiteralPrivate<T> for Literal<T> {
    fn __new(value: T, file: &str, line: u32, column: u32) -> Self {
        // Leak the string to get 'static lifetime (acceptable for tests)
        let file_static: &'static str = Box::leak(file.to_string().into_boxed_str());
        let mutex_ref = crate::registry::get_or_create(value, file_static, line, column);
        Literal::from_guard(mutex_ref.lock())
    }
}

impl<T: Value + 'static> Deref for Literal<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.literal
    }
}

impl<T: Value + 'static> std::ops::DerefMut for Literal<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.literal
    }
}

impl<T: Value + 'static> Drop for Literal<T> {
    /// Write changes to source file when the literal is dropped.
    ///
    /// # Error Handling Policy
    ///
    /// Since `Drop` cannot return errors, we use the following policy:
    ///
    /// 1. **Verification failures** (Verify mode): **Panic** - These are test failures
    ///    and should halt execution immediately to prevent false passing tests.
    ///
    /// 2. **Index resolution failures**: **Silent** - May occur with non-existent files
    ///    in test scenarios. We can't panic here as it would break legitimate use cases.
    ///
    /// 3. **Write failures** (Write mode): **Silent** - File I/O errors in Drop must be
    ///    silent to avoid panics. The background flush thread will retry. Users can also
    ///    call `.flush()` explicitly for error handling.
    ///
    /// 4. **Not running under cargo**: **Silent** - Safety measure to prevent accidental
    ///    writes outside of development environment.
    ///
    /// For explicit error handling, use [`LiteralExt::flush()`](crate::LiteralExt::flush)
    /// which returns `Result` instead of silently ignoring errors.
    fn drop(&mut self) {
        // Check if the value was mutated (via DerefMut or direct .literal assignment)
        // Compare using PartialEq
        if self.original != self.literal {
            // Value was mutated - mark as dirty for background flush
            crate::dirty::mark_dirty(&self.guard.file, self.guard.line, self.guard.column);

            // Trigger write based on mode
            let mode = crate::runtime::get_mode();

            // In Memory mode, update the guard but don't write to disk
            if mode == crate::runtime::Mode::Memory {
                self.guard.value = self.literal.clone();
                return;
            }

            // Skip writes in Reject mode
            if mode.should_reject_write() {
                return;
            }

            // For Verify or Write modes, we need file access
            if mode.needs_file_access() {
                // Resolve the index (if not already resolved)
                // Silent failure: May occur with non-existent files in tests
                if let Err(_) = self.guard.resolve_index() {
                    // Silently skip if we can't resolve the index
                    return;
                }

                // In Verify mode, verify that the value matches the source
                if mode == crate::runtime::Mode::Verify {
                    // Sync the public literal field back to guard for verification
                    self.guard.value = self.literal.clone();
                    // Verify - this WILL panic if there's a mismatch (intentional for test failures)
                    if let Err(e) = self.guard.verify_source(&self.guard.value) {
                        panic!(
                            "Literal verification failed at {}:{}:{}\\n{}",
                            self.guard.file.display(),
                            self.guard.line,
                            self.guard.column,
                            e
                        );
                    }
                    return;
                }

                // In Write mode, update the source
                if mode.can_write() {
                    // Check if we're running under cargo (safety measure)
                    if !is_running_under_cargo() {
                        // Silent failure: prevents writes outside development environment
                        return;
                    }

                    // Sync the public literal field back to the guard before writing
                    self.guard.value = self.literal.clone();

                    // Silent failure: Drop cannot return errors
                    // Background flush thread will retry failed writes
                    if self.guard.update_source(&self.guard.value).is_ok() {
                        // Clear dirty flag after successful write
                        crate::dirty::clear_dirty(&self.guard.file, self.guard.line, self.guard.column);
                    }
                }
            }
        }
    }
}

// Blanket trait implementations to make Literal<T> transparent

impl<T: Value + 'static> AsRef<T> for Literal<T> {
    fn as_ref(&self) -> &T {
        &self.literal
    }
}

impl<T: Value + 'static> std::borrow::Borrow<T> for Literal<T> {
    fn borrow(&self) -> &T {
        &self.literal
    }
}

impl<T: Value + std::fmt::Display + 'static> std::fmt::Display for Literal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.literal, f)
    }
}

impl<T: Value + 'static> Clone for Literal<T> {
    fn clone(&self) -> Self {
        // Return another smart pointer to the SAME registry entry
        use crate::LiteralPrivate;
        Literal::__new(
            self.guard.value.clone(),
            self.guard.file.to_str().unwrap(),
            self.guard.line,
            self.guard.column,
        )
    }
}

impl<T: Value + std::fmt::Debug + 'static> std::fmt::Debug for Literal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Literal")
            .field("literal", &self.literal)
            .finish()
    }
}

/// Create a self-modifying value that can update its source code.
///
/// The macro captures the source location and returns an [`Literal<T>`] that
/// holds a lock to the underlying value. The lock is held until the value
/// is dropped.
///
/// # Example
///
/// ```no_run
/// use jeb_literal::literal;
///
/// let mut counter = literal!(0u32);
/// let current = *counter;  // Single dereference
/// counter.literal = current + 1;
/// // In Write mode, the source file is updated
/// // Lock is released when counter goes out of scope
/// ```
///
/// # Requirements
///
/// The value type must implement the `Bake` trait from the `databake` crate.
///
/// # Returns
///
/// An [`Literal<T>`] that holds the lock and derefs to `&T`.
/// The same underlying value is returned for all calls from the same source location.
///
/// # Default Values
///
/// When called without arguments, uses `Default::default()`:
/// ```no_run
/// use jeb_literal::literal;
///
/// let counter: jeb_literal::Literal<u32> = literal!();  // Uses 0u32 (default)
/// ```
#[macro_export]
macro_rules! literal {
    () => {{
        $crate::Literal::from_guard($crate::registry::get_or_create(
            ::std::default::Default::default(),
            file!(),
            line!(),
            column!()
        ).lock())
    }};
    ($value:expr) => {{
        $crate::Literal::from_guard($crate::registry::get_or_create($value, file!(), line!(), column!()).lock())
    }};
}
