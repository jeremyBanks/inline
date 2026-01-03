use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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

/// Shared state across threads - the formatted source code is the source of truth
#[derive(Clone)]
struct SharedState {
    /// The source code (formatted with prettyplease)
    source: String,
    /// Version counter - incremented on every modification
    version: u64,
    /// The source code as it exists on disk (for detecting external modifications)
    /// Updated only when we read from or write to disk
    disk_source: String,
}

/// Thread-local cached AST and index mapping
struct CachedState {
    /// Parsed AST (cached to avoid re-parsing on every access)
    ast: syn::File,
    /// Maps (line, column) positions to stable indices
    position_to_index: HashMap<(u32, u32), usize>,
    /// Version this cache is based on
    version: u64,
}

// Global registry of FileStates (one per source file)
// Uses Arc so FileState can be shared across threads
static FILE_STATES: Lazy<RwLock<HashMap<PathBuf, FileState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

// Thread-local cache: each thread keeps its own parsed AST
thread_local! {
    static CACHE: RefCell<HashMap<PathBuf, CachedState>> = RefCell::new(HashMap::new());
}

/// Represents the shared state for a single source file
/// The key insight: String is Send+Sync, so we use it as the source of truth.
/// Each thread maintains a thread-local parsed AST for efficiency.
#[derive(Clone)]
pub struct FileState {
    /// Shared source of truth (formatted source code + version counter)
    shared: Arc<RwLock<SharedState>>,
    /// Path to this file (needed for caching)
    path: PathBuf,
}

impl FileState {
    /// Load and parse a source file
    pub fn load(path: &Path) -> Result<Self, io::Error> {
        let source = fs::read_to_string(path)?;

        // Validate that it parses
        syn::parse_file(&source).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse Rust file: {}", e),
            )
        })?;

        Ok(FileState {
            shared: Arc::new(RwLock::new(SharedState {
                source: source.clone(),
                version: 0,
                disk_source: source,
            })),
            path: path.to_path_buf(),
        })
    }

    /// Get a thread-local cached AST, re-parsing if the shared version has changed
    fn get_cached_ast(&self) -> Result<(syn::File, HashMap<(u32, u32), usize>), io::Error> {
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();

            // Get current version from shared state
            let shared = self.shared.read();
            let current_version = shared.version;
            let source = shared.source.clone();
            drop(shared); // Release read lock immediately

            // Check if we have a valid cached version
            if let Some(cached) = cache.get(&self.path) {
                if cached.version == current_version {
                    // Cache hit! Return cached AST and index map
                    return Ok((cached.ast.clone(), cached.position_to_index.clone()));
                }
            }

            // Cache miss or stale - need to re-parse
            let ast = syn::parse_file(&source).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse Rust file: {}", e),
                )
            })?;
            let position_to_index = Self::build_index_map(&ast);

            // Update thread-local cache
            cache.insert(
                self.path.clone(),
                CachedState {
                    ast: ast.clone(),
                    position_to_index: position_to_index.clone(),
                    version: current_version,
                },
            );

            Ok((ast, position_to_index))
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
    pub fn get_index(&self, line: u32, column: u32) -> Result<usize, io::Error> {
        let (_, position_to_index) = self.get_cached_ast()?;
        position_to_index
            .get(&(line, column))
            .copied()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "No litter! macro found at {}:{}:{}",
                        self.path.display(),
                        line,
                        column
                    ),
                )
            })
    }

    /// Update the macro at the given index with new tokens
    /// CRITICAL: Holds write lock for the entire operation to prevent concurrent modifications
    pub fn update_macro_by_index(
        &self,
        index: usize,
        new_tokens: proc_macro2::TokenStream,
    ) -> Result<(), String> {
        // ACQUIRE WRITE LOCK - blocks all other threads from reading or writing
        let mut shared = self.shared.write();

        // Parse the current source
        let mut ast = syn::parse_file(&shared.source)
            .map_err(|e| format!("Failed to parse source: {}", e))?;

        // Find and update the target macro
        let mut updater = IndexedMacroUpdater {
            target_index: index,
            current_index: 0,
            new_tokens,
            found: false,
        };

        updater.visit_file_mut(&mut ast);

        if !updater.found {
            return Err(format!("Could not find litter! macro at index {}", index));
        }

        // Format the modified AST
        let new_source = prettyplease::unparse(&ast);

        // Update shared state (increments version)
        shared.source = new_source;
        shared.version += 1;
        let new_version = shared.version;

        // Update thread-local cache with the new AST
        let position_to_index = Self::build_index_map(&ast);
        CACHE.with(|cache| {
            cache.borrow_mut().insert(
                self.path.clone(),
                CachedState {
                    ast,
                    position_to_index,
                    version: new_version,
                },
            );
        });

        // Write lock is released here when `shared` goes out of scope
        Ok(())
    }

    /// Get the current tokens of a litter macro at the given index
    pub fn get_macro_tokens(&self, index: usize) -> Result<proc_macro2::TokenStream, String> {
        let (ast, _) = self
            .get_cached_ast()
            .map_err(|e| format!("Failed to get AST: {}", e))?;

        let mut reader = IndexedMacroReader {
            target_index: index,
            current_index: 0,
            tokens: None,
        };

        use syn::visit::Visit;
        reader.visit_file(&ast);

        reader
            .tokens
            .ok_or_else(|| format!("Could not find litter! macro at index {}", index))
    }

    /// Write the current shared source to disk
    /// Then runs cargo fmt on the file to match project's rustfmt.toml
    ///
    /// IMPORTANT: Before writing, this verifies the file hasn't been modified by another process.
    /// If the file on disk differs from our expected state, this panics to prevent data loss.
    pub fn write_to_disk(&self) -> Result<(), io::Error> {
        // First, re-read the file from disk to detect concurrent modifications
        let current_disk_content = fs::read_to_string(&self.path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to read file before write (checking for concurrent modifications): {}",
                    e
                ),
            )
        })?;

        // Acquire write lock for the comparison and write
        let mut shared = self.shared.write();

        // Verify the file hasn't been modified by another process
        if current_disk_content != shared.disk_source {
            panic!(
                "CONCURRENT MODIFICATION DETECTED!\n\
                 File: {}\n\
                 \n\
                 Another process has modified this file since we loaded it.\n\
                 This is unsafe and could cause data loss.\n\
                 \n\
                 Expected content length: {} bytes\n\
                 Actual content length: {} bytes\n\
                 \n\
                 To avoid this error:\n\
                 - Run only one process that modifies this file at a time\n\
                 - Or use proper inter-process coordination\n",
                self.path.display(),
                shared.disk_source.len(),
                current_disk_content.len(),
            );
        }

        // Safe to write - no concurrent modification detected
        fs::write(&self.path, &shared.source)?;

        // Update our record of what's on disk
        shared.disk_source = shared.source.clone();

        drop(shared); // Release write lock before running cargo fmt

        // Run cargo fmt on this specific file to match project's formatting rules
        // This ensures stability with user running `cargo fmt` later
        if let Some(path_str) = self.path.to_str() {
            match std::process::Command::new("cargo")
                .args(["fmt", "--", path_str])
                .output()
            {
                Ok(output) if !output.status.success() => {
                    eprintln!(
                        "Warning: cargo fmt failed for {}: {}",
                        self.path.display(),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Warning: could not run cargo fmt for {}: {}",
                        self.path.display(),
                        e
                    );
                }
                _ => {
                    // cargo fmt succeeded - re-read the file to update our disk_source
                    // (cargo fmt may have reformatted the file)
                    // IMPORTANT: We do NOT update shared.source here!
                    // shared.source remains the prettyplease output, which is what
                    // all line/column positions are based on. disk_source tracks
                    // what's actually on disk (after cargo fmt).
                    if let Ok(formatted_content) = fs::read_to_string(&self.path) {
                        let mut shared = self.shared.write();
                        shared.disk_source = formatted_content;
                    }
                }
            }
        }

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

/// Get or load the FileState for a given path
fn get_or_load_file_state(path: &Path) -> Result<FileState, io::Error> {
    let states = FILE_STATES.read();

    // Fast path: already loaded
    if let Some(state) = states.get(path) {
        return Ok(state.clone());
    }

    drop(states); // Release read lock

    // Slow path: need to load
    let mut states = FILE_STATES.write();

    // Double-check in case another thread loaded it while we were waiting
    if let Some(state) = states.get(path) {
        return Ok(state.clone());
    }

    // Actually load the file
    let state = FileState::load(path)?;
    states.insert(path.to_path_buf(), state.clone());
    Ok(state)
}

/// Get the stable index for a litter macro at the given position
pub fn get_macro_index(path: &Path, line: u32, column: u32) -> Result<usize, io::Error> {
    let state = get_or_load_file_state(path)?;
    state.get_index(line, column)
}

/// Update a litter macro by its stable index
/// This only updates the in-memory shared state.
/// To persist to disk, you must call write_to_disk separately.
pub fn update_macro_by_index(
    path: &Path,
    index: usize,
    new_tokens: proc_macro2::TokenStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = get_or_load_file_state(path)?;
    state.update_macro_by_index(index, new_tokens)?;
    Ok(())
}

/// Write the current state of a file to disk
pub fn write_to_disk(path: &Path) -> Result<(), io::Error> {
    let state = get_or_load_file_state(path)?;
    state.write_to_disk()
}

/// Convenience function: update a litter macro at the given position
/// This combines get_macro_index, update_macro_by_index, and write_to_disk
pub fn update_source_file(
    path: &Path,
    line: u32,
    column: u32,
    new_tokens: proc_macro2::TokenStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let index = get_macro_index(path, line, column)?;
    update_macro_by_index(path, index, new_tokens)?;
    write_to_disk(path)?;
    Ok(())
}

/// Get the current tokens of a litter macro by its stable index
pub fn get_macro_tokens_by_index(
    path: &Path,
    index: usize,
) -> Result<proc_macro2::TokenStream, Box<dyn std::error::Error>> {
    let state = get_or_load_file_state(path)?;
    state.get_macro_tokens(index).map_err(|e| e.into())
}
