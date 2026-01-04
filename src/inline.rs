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
    value: T,
    file: PathBuf,
    line: u32,
    column: u32,
    /// Stable index into the file's literal macros (resolved lazily)
    /// This never changes even if line numbers shift!
    macro_index: Option<usize>,
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
    fn resolve_index(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
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
        // Compare by baked tokens, not by PartialEq
        // This way we only depend on Bake trait and detect actual semantic changes
        let env = databake::CrateEnv::default();
        let old_tokens = self.value.bake(&env).to_string();
        let new_tokens = new_value.bake(&env).to_string();

        if old_tokens == new_tokens {
            return; // No change needed - baked representation is identical
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
    fn verify_source(&self, new_value: &T) -> Result<(), Box<dyn std::error::Error>> {
        // Index must be resolved by now
        let index = self
            .macro_index
            .expect("Index should be resolved before calling verify_source");

        // Get the current tokens from the source file
        let current_tokens = crate::runtime::get_macro_tokens_by_index(&self.file, index)?;

        // Bake the new value to Rust code
        let env = databake::CrateEnv::default();
        let expected_tokens = new_value.bake(&env);

        // Normalize both token streams to strings for comparison
        let current_str = current_tokens.to_string();
        let expected_str = expected_tokens.to_string();

        if current_str != expected_str {
            return Err(format!(
                "Value mismatch!\n  Expected: {}\n  Found in source: {}",
                expected_str, current_str
            )
            .into());
        }

        Ok(())
    }

    /// Internal: update the source file with the new value
    fn update_source(&self, new_value: &T) -> Result<(), Box<dyn std::error::Error>> {
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
    std::env::var("CARGO").is_ok()
        || std::env::var("CARGO_MANIFEST_DIR").is_ok()
        || std::env::var("CARGO_PKG_NAME").is_ok()
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
/// counter.value = *counter + 1;
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
/// counter.value = 42;  // Direct field assignment, no * needed
/// // Value is automatically written on drop
/// ```
pub struct Literal<T: Value + 'static> {
    /// The current value. Mutating this field triggers write-on-drop.
    pub value: T,
    guard: parking_lot::MutexGuard<'static, LiteralInner<T>>,
    /// Clone of the original value when this Literal was created.
    /// Used in Drop to detect mutations.
    original: T,
}

impl<T: Value + 'static> Literal<T> {
    /// Create an Literal wrapper from a mutex guard
    #[doc(hidden)]
    pub fn from_guard(guard: parking_lot::MutexGuard<'static, LiteralInner<T>>) -> Self {
        // Clone the value twice: once for working copy, once for change detection
        let value = guard.value.clone();
        let original = guard.value.clone();
        Literal { value, guard, original }
    }

    /// Create a new Literal value for testing purposes
    ///
    /// This is equivalent to calling the macro, but allows specifying
    /// custom file/line/column values for testing.
    ///
    /// **Note:** For testing only. Leaks the file path string.
    #[doc(hidden)]
    pub fn __new(value: T, file: &str, line: u32, column: u32) -> Self {
        // Leak the string to get 'static lifetime (acceptable for tests)
        let file_static: &'static str = Box::leak(file.to_string().into_boxed_str());
        let mutex_ref = crate::registry::get_or_create(value, file_static, line, column);
        Literal::from_guard(mutex_ref.lock())
    }

    /// Get a reference to the current value
    ///
    /// Same as dereferencing, but explicit.
    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T: Value + 'static> Deref for Literal<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Value + 'static> std::ops::DerefMut for Literal<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Value + 'static> Drop for Literal<T> {
    fn drop(&mut self) {
        // Check if the value was mutated (via DerefMut or direct .value assignment)
        // Compare by baked tokens (same approach as set())
        let env = databake::CrateEnv::default();
        let original_tokens = self.original.bake(&env).to_string();
        let current_tokens = self.value.bake(&env).to_string();

        if original_tokens != current_tokens {
            // Value was mutated, trigger write
            let mode = crate::runtime::get_mode();

            // In Memory mode, update the guard but don't write to disk
            if mode == crate::runtime::Mode::Memory {
                self.guard.value = self.value.clone();
                return;
            }

            // Skip writes in Reject mode
            if mode.should_reject_write() {
                return;
            }

            // For Verify or Write modes, we need file access
            if mode.needs_file_access() {
                // Resolve the index (if not already resolved)
                if let Err(_) = self.guard.resolve_index() {
                    // Silently skip if we can't resolve the index
                    return;
                }

                // In Verify mode, verify that the value matches the source
                if mode == crate::runtime::Mode::Verify {
                    // Sync the public value back to guard for verification
                    self.guard.value = self.value.clone();
                    // Verify - this may panic if there's a mismatch
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
                    // Check if we're running under cargo
                    if !is_running_under_cargo() {
                        return;
                    }

                    // Sync the public value back to the guard before writing
                    self.guard.value = self.value.clone();

                    // Silently ignore errors in drop - we can't panic or return an error
                    let _ = self.guard.update_source(&self.guard.value);
                }
            }
        }
    }
}

impl<T: Value + std::fmt::Debug + 'static> std::fmt::Debug for Literal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Literal")
            .field("value", &self.value)
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
/// counter.value = current + 1;
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
