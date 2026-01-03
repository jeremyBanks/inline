use crate::literal::Literal;
use std::ops::Deref;
use std::path::PathBuf;

/// A self-modifying value that can update itself in source code
/// Uses a stable index to track its position in the AST
pub struct Litter<T: Literal> {
    value: T,
    file: PathBuf,
    line: u32,
    column: u32,
    /// Stable index into the file's litter macros (resolved lazily)
    /// This never changes even if line numbers shift!
    macro_index: Option<usize>,
}

impl<T: Literal> Litter<T> {
    /// Create a new Litter instance (called by the macro)
    /// Does NOT fail if the source file doesn't exist - that's only an error if you call set()
    #[doc(hidden)]
    pub fn __new(value: T, file: &str, line: u32, column: u32) -> Self {
        Litter {
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
                    "Failed to find litter! macro at {}:{}:{}\n{}",
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
                    "Litter verification failed at {}:{}:{}\n{}",
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
                     Hint: Run with 'cargo run' or 'cargo test', or use LITTER_MODE=memory",
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

        // Write to disk (this runs cargo fmt as well)
        crate::runtime::write_to_disk(&self.file)?;

        Ok(())
    }
}

impl<T: Literal> Deref for Litter<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Literal + std::fmt::Debug> std::fmt::Debug for Litter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Litter")
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

/// Macro to create a Litter instance
#[macro_export]
macro_rules! litter {
    ($value:expr) => {{
        $crate::Litter::__new($value, file!(), line!(), column!())
    }};
}
