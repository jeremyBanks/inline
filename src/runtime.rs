use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use syn::visit_mut::{self, VisitMut};

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Reads source files and verifies set() values match what's written
    /// Panics if there's a mismatch (snapshot testing behavior)
    /// DEFAULT IN TESTS
    Verify,
    /// Actually writes changes to source files
    /// Fails if files can't be found or litter macros missing at expected positions
    /// DEFAULT OUTSIDE TESTS (self-modifying code!)
    Write,
    /// Changes in memory only, never writes to disk
    /// Must be explicitly enabled via LITTER_MODE=memory
    Memory,
    /// Rejects any attempt to write, always fails
    /// Must be explicitly enabled via LITTER_MODE=reject
    Reject,
}

impl Mode {
    pub fn needs_file_access(self) -> bool {
        matches!(self, Mode::Verify | Mode::Write)
    }

    pub fn can_write(self) -> bool {
        matches!(self, Mode::Write)
    }

    pub fn should_reject_write(self) -> bool {
        matches!(self, Mode::Reject)
    }

    fn default_for_context() -> Self {
        // In tests: default to Verify (snapshot testing)
        // Outside tests: default to Write (self-modifying code)
        #[cfg(test)]
        return Mode::Verify;

        #[cfg(not(test))]
        return Mode::Write;
    }
}

/// Get the current mode by checking environment variable
///
/// Modes (set via LITTER_MODE environment variable):
/// - "verify": Verify values match source (DEFAULT IN TESTS)
/// - "write": Write changes to source files (DEFAULT OUTSIDE TESTS)
/// - "memory": Changes in memory only (opt-in only)
/// - "reject": Reject any write attempts (opt-in only)
///
/// Examples:
///   LITTER_MODE=write cargo test     # Update all snapshots
///   cargo test                        # Verify snapshots (default in tests)
///   cargo run                         # Self-modifying mode (default outside tests)
///   LITTER_MODE=memory cargo run     # Run without file writes
pub fn get_mode() -> Mode {
    if let Ok(mode_str) = env::var("LITTER_MODE") {
        return match mode_str.to_lowercase().as_str() {
            "write" | "update" => Mode::Write,
            "verify" => Mode::Verify,
            "memory" => Mode::Memory,
            "reject" => Mode::Reject,
            _ => {
                eprintln!("Warning: Unknown LITTER_MODE='{}', using default. Valid: write, verify, memory, reject", mode_str);
                Mode::default_for_context()
            }
        };
    }

    Mode::default_for_context()
}

/// Convenience constant for accessing mode
pub static MODE: Lazy<Mode> = Lazy::new(get_mode);

// Thread-local state: one FileState per source file
// The AST is kept in memory and mutated in place - never re-parsed!
// We use thread_local! because syn::File contains non-Send types (proc_macro::Span)
thread_local! {
    static FILE_STATES: RefCell<HashMap<PathBuf, FileState>> = RefCell::new(HashMap::new());
}

/// Represents the parsed AST and index mapping for a single source file
/// The key insight: we parse once, keep the AST in memory, and use stable indices
pub struct FileState {
    /// The AST is kept in memory and mutated in place
    ast: RwLock<syn::File>,
    /// Maps (line, column) positions to stable indices
    /// Built on first load, never changes
    position_to_index: HashMap<(u32, u32), usize>,
}

impl FileState {
    /// Load and parse a source file, building the position->index mapping
    pub fn load(path: &Path) -> Result<Self, io::Error> {
        let source = fs::read_to_string(path)?;
        let ast = syn::parse_file(&source).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse Rust file: {}", e),
            )
        })?;

        // Build the initial position->index mapping
        let position_to_index = Self::build_index_map(&ast);

        Ok(FileState {
            ast: RwLock::new(ast),
            position_to_index,
        })
    }

    /// Build a map from (line, column) to macro index by traversing the AST
    /// Indices are assigned in AST traversal order and never change
    fn build_index_map(ast: &syn::File) -> HashMap<(u32, u32), usize> {
        use syn::visit::Visit;

        struct IndexBuilder {
            map: HashMap<(u32, u32), usize>,
            current_index: usize,
        }

        impl<'ast> Visit<'ast> for IndexBuilder {
            fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
                let is_litter = if let Some(segment) = node.mac.path.segments.last() {
                    segment.ident == "litter"
                } else {
                    false
                };

                if is_litter {
                    let span = node.mac.path.segments.last().unwrap().ident.span();
                    let start = span.start();
                    let pos = (start.line as u32, start.column as u32);
                    self.map.insert(pos, self.current_index);
                    self.current_index += 1;
                }

                syn::visit::visit_expr_macro(self, node);
            }

            fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
                let is_litter = if let Some(segment) = node.mac.path.segments.last() {
                    segment.ident == "litter"
                } else {
                    false
                };

                if is_litter {
                    let span = node.mac.path.segments.last().unwrap().ident.span();
                    let start = span.start();
                    let pos = (start.line as u32, start.column as u32);
                    self.map.insert(pos, self.current_index);
                    self.current_index += 1;
                }

                syn::visit::visit_stmt_macro(self, node);
            }
        }

        let mut builder = IndexBuilder {
            map: HashMap::new(),
            current_index: 0,
        };
        builder.visit_file(ast);
        builder.map
    }

    /// Get the stable index for a macro at the given position
    pub fn get_index(&self, line: u32, column: u32) -> Option<usize> {
        self.position_to_index.get(&(line, column)).copied()
    }

    /// Update the macro at the given index with new tokens
    /// This is the core operation - we find the Nth litter! macro and update it
    pub fn update_macro_by_index(
        &self,
        index: usize,
        new_tokens: proc_macro2::TokenStream,
    ) -> Result<(), String> {
        let mut ast = self.ast.write();

        let mut updater = IndexedMacroUpdater {
            target_index: index,
            current_index: 0,
            new_tokens,
            found: false,
        };

        updater.visit_file_mut(&mut *ast);

        if !updater.found {
            return Err(format!("Could not find litter! macro at index {}", index));
        }

        Ok(())
    }

    /// Get the current tokens of a litter macro at the given index
    pub fn get_macro_tokens(&self, index: usize) -> Result<proc_macro2::TokenStream, String> {
        let ast = self.ast.read();

        let mut reader = IndexedMacroReader {
            target_index: index,
            current_index: 0,
            tokens: None,
        };

        use syn::visit::Visit;
        reader.visit_file(&*ast);

        reader.tokens.ok_or_else(|| format!("Could not find litter! macro at index {}", index))
    }

    /// Write the current AST back to disk
    pub fn write_to_disk(&self, path: &Path) -> Result<(), io::Error> {
        let ast = self.ast.read();
        let formatted = prettyplease::unparse(&*ast);
        fs::write(path, formatted)?;

        Ok(())
    }
}

/// Visitor that updates the Nth litter! macro (by index)
/// Key insight: we count macros in traversal order, which is stable
struct IndexedMacroUpdater {
    target_index: usize,
    current_index: usize,
    new_tokens: proc_macro2::TokenStream,
    found: bool,
}

impl IndexedMacroUpdater {
    fn try_update_macro(&mut self, mac: &mut syn::Macro) {
        if self.found {
            return;
        }

        let is_litter = if let Some(segment) = mac.path.segments.last() {
            segment.ident == "litter"
        } else {
            false
        };

        if is_litter {
            if self.current_index == self.target_index {
                // Found our target!
                mac.tokens = self.new_tokens.clone();
                self.found = true;
            }
            self.current_index += 1;
        }
    }
}

impl VisitMut for IndexedMacroUpdater {
    fn visit_expr_macro_mut(&mut self, node: &mut syn::ExprMacro) {
        self.try_update_macro(&mut node.mac);
        visit_mut::visit_expr_macro_mut(self, node);
    }

    fn visit_stmt_macro_mut(&mut self, node: &mut syn::StmtMacro) {
        self.try_update_macro(&mut node.mac);
        visit_mut::visit_stmt_macro_mut(self, node);
    }
}

/// Visitor that reads the Nth litter! macro (by index)
struct IndexedMacroReader {
    target_index: usize,
    current_index: usize,
    tokens: Option<proc_macro2::TokenStream>,
}

impl IndexedMacroReader {
    fn try_read_macro(&mut self, mac: &syn::Macro) {
        if self.tokens.is_some() {
            return;
        }

        let is_litter = if let Some(segment) = mac.path.segments.last() {
            segment.ident == "litter"
        } else {
            false
        };

        if is_litter {
            if self.current_index == self.target_index {
                // Found our target!
                self.tokens = Some(mac.tokens.clone());
            }
            self.current_index += 1;
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for IndexedMacroReader {
    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        self.try_read_macro(&node.mac);
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
        self.try_read_macro(&node.mac);
        syn::visit::visit_stmt_macro(self, node);
    }
}

/// Get the stable index for a litter macro at the given position
pub fn get_macro_index(path: &Path, line: u32, column: u32) -> Result<usize, io::Error> {
    FILE_STATES.with(|states| {
        let mut states = states.borrow_mut();

        // Load the file if not already loaded
        if !states.contains_key(path) {
            let state = FileState::load(path)?;
            states.insert(path.to_path_buf(), state);
        }

        let state = states.get(path).unwrap();
        state.get_index(line, column)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::NotFound,
                format!("No litter! macro found at {}:{}:{}", path.display(), line, column)
            ))
    })
}

/// Update a litter macro by its stable index
pub fn update_macro_by_index(
    path: &Path,
    index: usize,
    new_tokens: proc_macro2::TokenStream,
) -> Result<(), Box<dyn std::error::Error>> {
    FILE_STATES.with(|states| {
        let states = states.borrow();

        let state = states.get(path)
            .ok_or("File state not found - was get_macro_index called first?")?;

        state.update_macro_by_index(index, new_tokens)?;
        state.write_to_disk(path)?;

        Ok(())
    })
}

/// Convenience function: update a litter macro at the given position
/// This combines get_macro_index and update_macro_by_index
pub fn update_source_file(
    path: &Path,
    line: u32,
    column: u32,
    new_tokens: proc_macro2::TokenStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let index = get_macro_index(path, line, column)?;
    update_macro_by_index(path, index, new_tokens)?;
    Ok(())
}

/// Get the current tokens of a litter macro by its stable index
pub fn get_macro_tokens_by_index(
    path: &Path,
    index: usize,
) -> Result<proc_macro2::TokenStream, Box<dyn std::error::Error>> {
    FILE_STATES.with(|states| {
        let states = states.borrow();

        let state = states.get(path)
            .ok_or("File state not found - was get_macro_index called first?")?;

        state.get_macro_tokens(index)
            .map_err(|e| e.into())
    })
}
