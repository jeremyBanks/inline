use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use syn::visit_mut::{self, VisitMut};

#[derive(Clone, Debug, Copy, Default)]
pub enum Mode {
    #[default]
    /// don't read the source files at all.
    Inactive,
    /// reads values from source files to verify that the current values
    /// round-trip successfully, but don't write anything.
    Verify,
    /// writes values back if they aren't equal to the expected value.
    Update,
}

impl Mode {
    pub fn read(self) -> bool {
        matches!(self, Mode::Verify | Mode::Update)
    }

    pub fn write(self) -> bool {
        matches!(self, Mode::Update)
    }

    pub fn inactive(self) -> bool {
        matches!(self, Mode::Inactive)
    }
}

/// Get the current mode by checking environment variables
pub fn get_mode() -> Mode {
    if env::var("LITTER_UPDATE").is_ok() {
        Mode::Update
    } else if env::var("LITTER_VERIFY").is_ok() {
        Mode::Verify
    } else {
        Mode::default()
    }
}

/// Convenience constant for accessing mode
pub static MODE: Lazy<Mode> = Lazy::new(get_mode);

/// Per-file locks to prevent concurrent writes to the same file
static FILE_LOCKS: Lazy<Mutex<HashMap<PathBuf, ()>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Acquire a lock for a specific file path
fn lock_file(path: &Path) {
    let mut locks = FILE_LOCKS.lock();
    // Insert if not present, this ensures we have an entry to lock on
    locks.entry(path.to_path_buf()).or_insert(());
    // Note: we're using the global mutex as the lock mechanism
    // This is simple but means only one file can be updated at a time
    // For a more sophisticated approach, we'd use per-file locks
}

/// Update a source file by replacing a litter! macro invocation
pub fn update_source_file(
    path: &Path,
    line: u32,
    column: u32,
    new_tokens: proc_macro2::TokenStream,
) -> Result<(), Box<dyn std::error::Error>> {
    // Lock the file
    lock_file(path);
    let _file_lock = FILE_LOCKS.lock();

    // Read the source file
    let source = fs::read_to_string(path)?;

    // Parse it
    let mut ast = syn::parse_file(&source).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse Rust file: {}", e),
        )
    })?;

    // Find and replace the macro
    let mut replacer = MacroReplacer {
        target_line: line as usize,
        target_column: column as usize,
        new_tokens,
        found: false,
    };

    replacer.visit_file_mut(&mut ast);

    if !replacer.found {
        return Err(format!(
            "Could not find litter! macro at line {}, column {}",
            line, column
        )
        .into());
    }

    // Format and write back
    let formatted = prettyplease::unparse(&ast);
    fs::write(path, formatted)?;

    Ok(())
}

/// Visitor that finds and replaces a specific litter! macro invocation
struct MacroReplacer {
    target_line: usize,
    target_column: usize,
    new_tokens: proc_macro2::TokenStream,
    found: bool,
}

impl MacroReplacer {
    fn try_replace_macro(&mut self, mac: &mut syn::Macro) {
        if self.found {
            return;
        }

        // Check if this is a litter macro (might be just "litter" or "litter::litter")
        let is_litter = if let Some(segment) = mac.path.segments.last() {
            segment.ident == "litter"
        } else {
            false
        };

        if is_litter {
            // Get the span of the last segment (the actual "litter" identifier)
            let ident = &mac.path.segments.last().unwrap().ident;
            let span = ident.span();
            let start = span.start();

            if start.line == self.target_line && start.column == self.target_column {
                // Found it! Replace the tokens
                mac.tokens = self.new_tokens.clone();
                self.found = true;
            }
        }
    }
}

impl VisitMut for MacroReplacer {
    fn visit_expr_macro_mut(&mut self, node: &mut syn::ExprMacro) {
        self.try_replace_macro(&mut node.mac);
        visit_mut::visit_expr_macro_mut(self, node);
    }

    fn visit_stmt_macro_mut(&mut self, node: &mut syn::StmtMacro) {
        self.try_replace_macro(&mut node.mac);
        visit_mut::visit_stmt_macro_mut(self, node);
    }
}
