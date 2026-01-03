use crate::literal::Literal;
use std::ops::Deref;
use std::path::PathBuf;

/// Internal implementation of a self-modifying value.
///
/// This type wraps a value and provides the ability to update both the
/// in-memory value and its representation in the source code file.
///
/// **Note:** This is an internal type. Users should interact with the [`Inline`]
/// wrapper returned by the `inline!` macro instead.
#[doc(hidden)]
pub struct InlineInner<T: Literal> {
    value: T,
    file: PathBuf,
    line: u32,
    column: u32,
    /// Stable index into the file's inline macros (resolved lazily)
    /// This never changes even if line numbers shift!
    macro_index: Option<usize>,
}

impl<T: Literal> InlineInner<T> {
    /// Create a new InlineInner instance (called by the registry)
    /// Does NOT fail if the source file doesn't exist - that's only an error if you call set()
    pub(crate) fn new(value: T, file: &str, line: u32, column: u32) -> Self {
        InlineInner {
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
                    "Failed to find inline! macro at {}:{}:{}\n{}",
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
                    "Inline verification failed at {}:{}:{}\n{}",
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
                     Hint: Run with 'cargo run' or 'cargo test', or use INLINE_MODE=memory",
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

impl<T: Literal> Deref for InlineInner<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Literal + std::fmt::Debug> std::fmt::Debug for InlineInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineInner")
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
/// This type wraps a `MutexGuard` to an [`InlineInner<T>`] and provides
/// convenient access to the value with a single dereference.
///
/// Created via the [`inline!`](macro@crate::inline) macro. The lock is held
/// for the entire lifetime of this value.
///
/// # Example
///
/// ```no_run
/// use inline::inline;
///
/// let mut counter = inline!(0u32);
/// println!("Value: {}", *counter);  // Single deref
/// counter.set(*counter + 1);
/// ```
pub struct Inline<T: Literal + 'static> {
    guard: parking_lot::MutexGuard<'static, InlineInner<T>>,
}

impl<T: Literal + 'static> Inline<T> {
    /// Create an Inline wrapper from a mutex guard
    #[doc(hidden)]
    pub fn from_guard(guard: parking_lot::MutexGuard<'static, InlineInner<T>>) -> Self {
        Inline { guard }
    }

    /// Create a new Inline value for testing purposes
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
        Inline::from_guard(mutex_ref.lock())
    }

    /// Update the value and possibly persist to source file
    ///
    /// Delegates to [`InlineInner::set()`].
    pub fn set(&mut self, new_value: T) {
        self.guard.set(new_value);
    }

    /// Get a reference to the current value
    ///
    /// Same as dereferencing, but explicit.
    pub fn get(&self) -> &T {
        &*self.guard
    }
}

impl<T: Literal + 'static> Deref for Inline<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.guard  // Deref guard to InlineInner, then to T
    }
}

impl<T: Literal + std::fmt::Debug + 'static> std::fmt::Debug for Inline<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inline")
            .field("value", &**self)  // Double deref to get to T
            .finish()
    }
}

/// Create a self-modifying value that can update its source code.
///
/// The macro captures the source location and returns an [`Inline<T>`] that
/// holds a lock to the underlying value. The lock is held until the value
/// is dropped.
///
/// # Example
///
/// ```no_run
/// use inline::inline;
///
/// let mut counter = inline!(0u32);
/// let current = *counter;  // Single dereference
/// counter.set(current + 1);
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
/// An [`Inline<T>`] that holds the lock and derefs to `&T`.
/// The same underlying value is returned for all calls from the same source location.
#[macro_export]
macro_rules! inline {
    ($value:expr) => {{
        $crate::Inline::from_guard($crate::registry::get_or_create($value, file!(), line!(), column!()).lock())
    }};
}
