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

        let index = crate::runtime::get_macro_index(&self.file, self.line, self.column)
            .map_err(|e| format!(
                "Failed to find litter! macro at {}:{}:{}\n{}",
                self.file.display(), self.line, self.column, e
            ))?;

        self.macro_index = Some(index);
        Ok(index)
    }

    /// Get a reference to the current value
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Update the value and write it back to the source file
    pub fn set(&mut self, new_value: T) {
        if self.value == new_value {
            return; // No change needed
        }

        let old_value = self.value.clone();
        self.value = new_value.clone();

        let mode = crate::runtime::get_mode();

        // In Inactive mode, just change the value in memory
        if mode.inactive() {
            return;
        }

        // Resolve the index (lazily loads the file)
        // This is where we'll fail if the file doesn't exist or position is invalid
        if let Err(e) = self.resolve_index() {
            self.value = old_value; // Rollback
            panic!("Failed to access source file: {}", e);
        }

        // In Verify mode: check that the new value matches the source file
        if matches!(mode, crate::runtime::Mode::Verify) {
            if let Err(e) = self.verify_source(&new_value) {
                panic!("Litter verification failed at {}:{}:{}\n{}",
                    self.file.display(), self.line, self.column, e);
            }
            return;
        }

        // In Update mode: write changes to disk
        if !mode.write() {
            return;
        }

        // Try to update the source file
        if let Err(e) = self.update_source(&new_value) {
            // Rollback on failure
            self.value = old_value;
            eprintln!("Warning: Failed to update source file: {}", e);
        }
    }

    /// Internal: verify that the new value matches what's in the source file
    fn verify_source(&self, new_value: &T) -> Result<(), Box<dyn std::error::Error>> {
        // Index must be resolved by now
        let index = self.macro_index.expect("Index should be resolved before calling verify_source");

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
            ).into());
        }

        Ok(())
    }

    /// Internal: update the source file with the new value
    fn update_source(&self, new_value: &T) -> Result<(), Box<dyn std::error::Error>> {
        // Index must be resolved by now
        let index = self.macro_index.expect("Index should be resolved before calling update_source");

        // Bake the value to Rust code
        let env = databake::CrateEnv::default();
        let baked_tokens = new_value.bake(&env);

        // Update using our stable index
        crate::runtime::update_macro_by_index(&self.file, index, baked_tokens)?;

        Ok(())
    }
}

impl<T: Literal> Deref for Litter<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Literal> PartialEq<T> for Litter<T> {
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

impl<T: Literal> Clone for Litter<T> {
    fn clone(&self) -> Self {
        Litter {
            value: self.value.clone(),
            file: self.file.clone(),
            line: self.line,
            column: self.column,
            macro_index: self.macro_index,
        }
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

/// Macro to create a Litter instance
#[macro_export]
macro_rules! litter {
    ($value:expr) => {{
        $crate::Litter::__new($value, file!(), line!(), column!())
    }};
}
