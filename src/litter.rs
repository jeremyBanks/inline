use crate::literal::Literal;
use std::ops::Deref;
use std::path::PathBuf;

/// A self-modifying value that can update itself in source code
/// Uses a stable index to track its position in the AST
pub struct Litter<T: Literal> {
    value: T,
    file: PathBuf,
    /// Stable index into the file's litter macros
    /// This never changes even if line numbers shift!
    macro_index: usize,
}

impl<T: Literal> Litter<T> {
    /// Create a new Litter instance (called by the macro)
    #[doc(hidden)]
    pub fn __new(value: T, file: &str, line: u32, column: u32) -> Self {
        let file_path = PathBuf::from(file);

        // Get the stable index for this macro position
        let macro_index = crate::runtime::get_macro_index(&file_path, line, column)
            .expect(&format!("No litter! macro found at {}:{}:{}", file, line, column));

        Litter {
            value,
            file: file_path,
            macro_index,
        }
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

        // Only write if we're in Update mode
        let mode = crate::runtime::get_mode();
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

    /// Internal: update the source file with the new value
    fn update_source(&self, new_value: &T) -> Result<(), Box<dyn std::error::Error>> {
        // Bake the value to Rust code
        let env = databake::CrateEnv::default();
        let baked_tokens = new_value.bake(&env);

        // Update using our stable index
        crate::runtime::update_macro_by_index(&self.file, self.macro_index, baked_tokens)?;

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
            macro_index: self.macro_index,
        }
    }
}

impl<T: Literal + std::fmt::Debug> std::fmt::Debug for Litter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Litter")
            .field("value", &self.value)
            .field("file", &self.file)
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
