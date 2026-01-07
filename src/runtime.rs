use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Check if we're running under cargo by looking for cargo-specific env vars.
///
/// Returns `true` if any of CARGO, CARGO_MANIFEST_DIR, or CARGO_PKG_NAME
/// environment variables are set, indicating the program was launched via cargo.
pub fn is_running_under_cargo() -> bool {
    env::var("CARGO").is_ok()
        || env::var("CARGO_MANIFEST_DIR").is_ok()
        || env::var("CARGO_PKG_NAME").is_ok()
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Reads source files and verifies mutated values match what's in source.
    /// Panics if there's a mismatch (snapshot testing behavior).
    /// DEFAULT IN TESTS
    Verify,
    /// Actually writes changes to source files
    /// Fails if files can't be found or literal macros missing at expected positions
    /// DEFAULT OUTSIDE TESTS (self-modifying code!)
    Write,
    /// Changes in memory only, never writes to disk
    /// Must be explicitly enabled via INLINE_MODE=memory
    Memory,
    /// Rejects any attempt to write, always fails
    /// Must be explicitly enabled via INLINE_MODE=reject
    Reject,
}

impl Mode {
    pub fn needs_file_access(self) -> bool {
        matches!(self, Mode::Verify | Mode::Write)
    }

    pub fn can_write(self) -> bool {
        // If no-write feature is enabled, never allow writes
        #[cfg(feature = "no-write")]
        {
            false
        }

        // If write feature is disabled, Write mode behaves like Memory mode
        #[cfg(all(not(feature = "no-write"), feature = "write"))]
        {
            matches!(self, Mode::Write)
        }

        #[cfg(all(not(feature = "no-write"), not(feature = "write")))]
        {
            false
        }
    }

    pub fn should_reject_write(self) -> bool {
        matches!(self, Mode::Reject)
    }

    fn default_for_context() -> Self {
        // In tests: default to Verify (snapshot testing)
        #[cfg(test)]
        return Mode::Verify;

        // Outside tests: default based on cargo detection
        #[cfg(not(test))]
        {
            // If running under cargo, default to Write mode
            // Otherwise default to Memory mode for safety
            if is_running_under_cargo() {
                Mode::Write
            } else {
                Mode::Memory
            }
        }
    }
}

/// Get the current mode by checking environment variable
///
/// Modes (set via INLINE_MODE environment variable):
/// - "verify": Verify values match source (DEFAULT IN TESTS)
/// - "write": Write changes to source files (DEFAULT OUTSIDE TESTS)
/// - "memory": Changes in memory only (opt-in only)
/// - "reject": Reject any write attempts (opt-in only)
///
/// Examples:
///   INLINE_MODE=write cargo test     # Update all snapshots
///   cargo test                        # Verify snapshots (default in tests)
///   cargo run                         # Self-modifying mode (default outside tests)
///   INLINE_MODE=memory cargo run     # Run without file writes
pub fn get_mode() -> Mode {
    if let Ok(mode_str) = env::var("INLINE_MODE") {
        return match mode_str.to_lowercase().as_str() {
            "write" | "update" => Mode::Write,
            "verify" => Mode::Verify,
            "memory" => Mode::Memory,
            "reject" => Mode::Reject,
            _ => {
                eprintln!("Warning: Unknown INLINE_MODE='{}', using default. Valid: write, verify, memory, reject", mode_str);
                Mode::default_for_context()
            }
        };
    }

    Mode::default_for_context()
}

/// Shared state across threads - the source code (with original formatting) is the source of truth
#[derive(Clone)]
struct SharedState {
    /// The source code (preserves original formatting via character-range splicing)
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

/// Type alias for the return value of get_cached_ast
type CachedAstResult = Result<(syn::File, HashMap<(u32, u32), usize>), io::Error>;

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
    ///
    /// IMPORTANT: Always parses from disk_source (the original file content), not from
    /// the modified source. This ensures compile-time (line, column) coordinates always
    /// resolve against the original source, even after runtime modifications.
    fn get_cached_ast(&self) -> CachedAstResult {
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();

            // Get current version and DISK source from shared state
            let shared = self.shared.read();
            let current_version = shared.version;
            let disk_source = shared.disk_source.clone();  // Parse from original, not modified source
            drop(shared); // Release read lock immediately

            // Check if we have a valid cached version
            if let Some(cached) = cache.get(&self.path) {
                if cached.version == current_version {
                    // Cache hit! Return cached AST and index map
                    return Ok((cached.ast.clone(), cached.position_to_index.clone()));
                }
            }

            // Cache miss or stale - need to re-parse from ORIGINAL disk source
            // This ensures (line, column) coordinates always match the original file
            let ast = syn::parse_file(&disk_source).map_err(|e| {
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

    /// Build a map from (line, column) to literal call index by traversing the AST
    /// Indices are assigned in AST traversal order and never change
    fn build_index_map(ast: &syn::File) -> HashMap<(u32, u32), usize> {
        use syn::visit::Visit;

        struct IndexBuilder {
            map: HashMap<(u32, u32), usize>,
            current_index: usize,
            /// When true, don't index macros (we're inside a function call's args)
            skip_macros: bool,
        }

        impl<'ast> Visit<'ast> for IndexBuilder {
            fn visit_expr(&mut self, node: &'ast syn::Expr) {
                use syn::spanned::Spanned;

                // Index function calls, method calls, and macro invocations by position.
                // When a macro like cell!() expands to cell(), #[track_caller]
                // reports the macro call site. So we need to index macros too.
                //
                // IMPORTANT: We skip macros inside function/method call arguments
                // (like vec![] in cell(vec![1,2])) to avoid indexing confusion.
                match node {
                    syn::Expr::Call(call) => {
                        // Index this function call
                        let start = call.func.span().start();
                        let pos = (start.line as u32, start.column as u32);
                        self.map.insert(pos, self.current_index);
                        self.current_index += 1;

                        // Recurse into function position (for chained calls like foo().bar())
                        self.visit_expr(&call.func);

                        // Recurse into args, but skip any macros found there
                        let was_skipping = self.skip_macros;
                        self.skip_macros = true;
                        for arg in &call.args {
                            self.visit_expr(arg);
                        }
                        self.skip_macros = was_skipping;
                        return;
                    }
                    syn::Expr::MethodCall(method) => {
                        // Index this method call
                        let start = method.receiver.span().start();
                        let pos = (start.line as u32, start.column as u32);
                        self.map.insert(pos, self.current_index);
                        self.current_index += 1;

                        // Recurse into receiver (for chained calls)
                        self.visit_expr(&method.receiver);

                        // Recurse into args, but skip any macros found there
                        let was_skipping = self.skip_macros;
                        self.skip_macros = true;
                        for arg in &method.args {
                            self.visit_expr(arg);
                        }
                        self.skip_macros = was_skipping;
                        return;
                    }
                    syn::Expr::Macro(mac) => {
                        // Only index macros if we're not inside a function call's args
                        if !self.skip_macros {
                            let start = mac.mac.path.span().start();
                            let pos = (start.line as u32, start.column as u32);
                            self.map.insert(pos, self.current_index);
                            self.current_index += 1;
                        }
                        // Don't recurse into macro tokens - they're opaque
                        return;
                    }
                    _ => {}
                }

                syn::visit::visit_expr(self, node);
            }
        }

        let mut builder = IndexBuilder {
            map: HashMap::new(),
            current_index: 0,
            skip_macros: false,
        };
        builder.visit_file(ast);
        builder.map
    }

    /// Get the stable index for a function call at the given position
    ///
    /// Note: column matching is flexible because Location::caller().column()
    /// returns the start of the function call. We match on line and find the
    /// closest function call on that line.
    pub fn get_index(&self, line: u32, column: u32) -> Result<usize, io::Error> {
        let (_, position_to_index) = self.get_cached_ast()?;

        // First try exact match
        if let Some(&index) = position_to_index.get(&(line, column)) {
            return Ok(index);
        }

        // If no exact match, find all function calls on the same line
        let calls_on_line: Vec<_> = position_to_index
            .iter()
            .filter(|((l, _c), _idx)| *l == line)
            .collect();

        if calls_on_line.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "No function call found on line {} in {}",
                    line,
                    self.path.display()
                ),
            ));
        }

        // If there's only one call on this line, use it
        if calls_on_line.len() == 1 {
            return Ok(*calls_on_line[0].1);
        }

        // Multiple calls on same line - find closest by column
        let closest = calls_on_line
            .iter()
            .min_by_key(|((_, c), _)| (*c as i32 - column as i32).abs())
            .unwrap();

        Ok(*closest.1)
    }

    /// Update the literal() call at the given index with new tokens
    /// CRITICAL: Holds write lock for the entire operation to prevent concurrent modifications
    ///
    /// IMPORTANT: Uses character-range splicing to preserve formatting!
    /// Only the function argument is replaced - everything else stays untouched.
    pub fn update_literal_by_index(
        &self,
        index: usize,
        new_tokens: proc_macro2::TokenStream,
    ) -> Result<(), String> {
        // ACQUIRE WRITE LOCK - blocks all other threads from reading or writing
        let mut shared = self.shared.write();

        // Get the source while holding the lock (avoid deadlock)
        let source = shared.source.clone();

        // Parse the current source to find the literal call
        let ast = syn::parse_file(&source)
            .map_err(|e| format!("Failed to parse source: {}", e))?;

        // Find the byte span of the target call's argument
        // Pass source as parameter to avoid deadlock
        let value_span = Self::find_literal_arg_span_static(&ast, index, &source)?;

        // Generate the new value string
        let new_value_str = new_tokens.to_string();

        // Perform character-range splicing to preserve formatting
        let mut new_source = String::with_capacity(source.len());
        new_source.push_str(&source[..value_span.start]);
        new_source.push_str(&new_value_str);
        new_source.push_str(&source[value_span.end..]);

        // Update shared state (increments version)
        shared.source = new_source;
        shared.version += 1;

        // Invalidate thread-local caches by incrementing version
        // They'll re-parse on next access

        // Write lock is released here when `shared` goes out of scope
        Ok(())
    }

    // Keep old name as alias for compatibility during transition
    pub fn update_macro_by_index(
        &self,
        index: usize,
        new_tokens: proc_macro2::TokenStream,
    ) -> Result<(), String> {
        self.update_literal_by_index(index, new_tokens)
    }

    /// Find the byte span of a call's argument in the source code (static method)
    /// Handles both function calls and macro invocations.
    fn find_literal_arg_span_static(
        ast: &syn::File,
        target_index: usize,
        source: &str,
    ) -> Result<std::ops::Range<usize>, String> {
        use syn::visit::Visit;

        struct SpanFinder {
            target_index: usize,
            current_index: usize,
            span: Option<(proc_macro2::LineColumn, proc_macro2::LineColumn)>,
            skip_macros: bool,
        }

        impl<'ast> Visit<'ast> for SpanFinder {
            fn visit_expr(&mut self, node: &'ast syn::Expr) {
                use syn::spanned::Spanned;

                // Must match the same traversal logic as IndexBuilder
                match node {
                    syn::Expr::Call(call) => {
                        // Check if this is our target - use LAST arg (trailing position)
                        if self.current_index == self.target_index {
                            if let Some(arg) = call.args.last() {
                                self.span = Some((arg.span().start(), arg.span().end()));
                            }
                        }
                        self.current_index += 1;

                        // Recurse into function position
                        self.visit_expr(&call.func);

                        // Recurse into args, but skip macros
                        let was_skipping = self.skip_macros;
                        self.skip_macros = true;
                        for arg in &call.args {
                            self.visit_expr(arg);
                        }
                        self.skip_macros = was_skipping;
                        return;
                    }
                    syn::Expr::MethodCall(method) => {
                        // Check if this is our target - replace RECEIVER
                        if self.current_index == self.target_index {
                            let receiver = &method.receiver;
                            self.span = Some((receiver.span().start(), receiver.span().end()));
                        }
                        self.current_index += 1;

                        // Recurse into receiver
                        self.visit_expr(&method.receiver);

                        // Recurse into args, but skip macros
                        let was_skipping = self.skip_macros;
                        self.skip_macros = true;
                        for arg in &method.args {
                            self.visit_expr(arg);
                        }
                        self.skip_macros = was_skipping;
                        return;
                    }
                    syn::Expr::Macro(mac) => {
                        if !self.skip_macros {
                            if self.current_index == self.target_index {
                                // For macros, parse tokens as expression for full span
                                let tokens = mac.mac.tokens.clone();
                                if !tokens.is_empty() {
                                    if let Ok(expr) = syn::parse2::<syn::Expr>(tokens) {
                                        self.span = Some((expr.span().start(), expr.span().end()));
                                    }
                                }
                            }
                            self.current_index += 1;
                        }
                        return;
                    }
                    _ => {}
                }

                syn::visit::visit_expr(self, node);
            }
        }

        let mut finder = SpanFinder {
            target_index,
            current_index: 0,
            span: None,
            skip_macros: false,
        };

        finder.visit_file(ast);

        let (start_lc, end_lc) = finder
            .span
            .ok_or_else(|| format!("Could not find call/macro at index {}", target_index))?;

        // Convert line/column to byte offsets
        let start_byte = Self::line_col_to_byte_static(source, start_lc.line, start_lc.column)?;
        let end_byte = Self::line_col_to_byte_static(source, end_lc.line, end_lc.column)?;

        Ok(start_byte..end_byte)
    }

    /// Convert line/column (1-indexed line, 0-indexed column) to byte offset (static method)
    fn line_col_to_byte_static(
        source: &str,
        line: usize,
        column: usize,
    ) -> Result<usize, String> {
        let mut current_line = 0;
        let mut byte_offset = 0;

        for (idx, ch) in source.char_indices() {
            if current_line + 1 == line {
                // We're on the target line
                let chars_on_line = source[byte_offset..].chars().take(column).count();
                if chars_on_line == column {
                    // Count bytes for 'column' characters
                    let bytes: usize = source[byte_offset..]
                        .chars()
                        .take(column)
                        .map(|c| c.len_utf8())
                        .sum();
                    return Ok(byte_offset + bytes);
                }
            }

            if ch == '\n' {
                current_line += 1;
                byte_offset = idx + 1;
            }
        }

        // Handle last line (no trailing newline)
        if current_line + 1 == line {
            let bytes: usize = source[byte_offset..]
                .chars()
                .take(column)
                .map(|c| c.len_utf8())
                .sum();
            return Ok(byte_offset + bytes);
        }

        Err(format!(
            "Line {} column {} is out of bounds (source has {} lines)",
            line,
            column,
            source.lines().count()
        ))
    }

    /// Get the current tokens of a literal() call's argument at the given index
    pub fn get_literal_tokens(&self, index: usize) -> Result<proc_macro2::TokenStream, String> {
        let (ast, _) = self
            .get_cached_ast()
            .map_err(|e| format!("Failed to get AST: {}", e))?;

        let mut reader = IndexedLiteralReader {
            target_index: index,
            current_index: 0,
            tokens: None,
            skip_macros: false,
        };

        use syn::visit::Visit;
        reader.visit_file(&ast);

        reader
            .tokens
            .ok_or_else(|| format!("Could not find function call at index {}", index))
    }

    // Keep old name as alias for compatibility
    pub fn get_macro_tokens(&self, index: usize) -> Result<proc_macro2::TokenStream, String> {
        self.get_literal_tokens(index)
    }

    /// Write the current shared source to disk.
    ///
    /// Character-range splicing preserves the original formatting, so no reformatting is needed.
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

        // NOTE: We do NOT run cargo fmt here!
        // Character-range splicing preserves the original formatting,
        // so there's no need to reformat the file.

        Ok(())
    }

    /// Replace the entire function call expression at the given index with new tokens.
    ///
    /// Unlike `update_literal_by_index` which only replaces the argument,
    /// this replaces the entire `func(arg)` expression with the replacement tokens.
    ///
    /// Used by `replace_me()` to substitute the whole call with the baked value.
    pub fn replace_expression_by_index(
        &self,
        index: usize,
        replacement: proc_macro2::TokenStream,
    ) -> Result<(), String> {
        // ACQUIRE WRITE LOCK
        let mut shared = self.shared.write();

        let source = shared.source.clone();

        // Parse to find the expression
        let ast = syn::parse_file(&source)
            .map_err(|e| format!("Failed to parse source: {}", e))?;

        // Find the byte span of the entire call expression
        let expr_span = Self::find_call_expression_span_static(&ast, index, &source)?;

        // Generate the replacement string
        let replacement_str = replacement.to_string();

        // Perform character-range splicing
        let mut new_source = String::with_capacity(source.len());
        new_source.push_str(&source[..expr_span.start]);
        new_source.push_str(&replacement_str);
        new_source.push_str(&source[expr_span.end..]);

        // Update shared state
        shared.source = new_source;
        shared.version += 1;

        Ok(())
    }

    /// Find the byte span of an entire function call or macro expression at the given index.
    fn find_call_expression_span_static(
        ast: &syn::File,
        target_index: usize,
        source: &str,
    ) -> Result<std::ops::Range<usize>, String> {
        use syn::visit::Visit;

        struct ExprSpanFinder {
            target_index: usize,
            current_index: usize,
            span: Option<(proc_macro2::LineColumn, proc_macro2::LineColumn)>,
            skip_macros: bool,
        }

        impl<'ast> Visit<'ast> for ExprSpanFinder {
            fn visit_expr(&mut self, node: &'ast syn::Expr) {
                use syn::spanned::Spanned;

                // Must match the same traversal logic as IndexBuilder
                match node {
                    syn::Expr::Call(call) => {
                        if self.current_index == self.target_index {
                            self.span = Some((call.span().start(), call.span().end()));
                        }
                        self.current_index += 1;

                        self.visit_expr(&call.func);

                        let was_skipping = self.skip_macros;
                        self.skip_macros = true;
                        for arg in &call.args {
                            self.visit_expr(arg);
                        }
                        self.skip_macros = was_skipping;
                        return;
                    }
                    syn::Expr::MethodCall(method) => {
                        if self.current_index == self.target_index {
                            self.span = Some((method.span().start(), method.span().end()));
                        }
                        self.current_index += 1;

                        self.visit_expr(&method.receiver);

                        let was_skipping = self.skip_macros;
                        self.skip_macros = true;
                        for arg in &method.args {
                            self.visit_expr(arg);
                        }
                        self.skip_macros = was_skipping;
                        return;
                    }
                    syn::Expr::Macro(mac) => {
                        if !self.skip_macros {
                            if self.current_index == self.target_index {
                                self.span = Some((mac.span().start(), mac.span().end()));
                            }
                            self.current_index += 1;
                        }
                        return;
                    }
                    _ => {}
                }

                syn::visit::visit_expr(self, node);
            }
        }

        let mut finder = ExprSpanFinder {
            target_index,
            current_index: 0,
            span: None,
            skip_macros: false,
        };

        finder.visit_file(ast);

        let (start_lc, end_lc) = finder
            .span
            .ok_or_else(|| format!("Could not find function call at index {}", target_index))?;

        // Convert line/column to byte offsets
        let start_byte = Self::line_col_to_byte_static(source, start_lc.line, start_lc.column)?;
        let end_byte = Self::line_col_to_byte_static(source, end_lc.line, end_lc.column)?;

        Ok(start_byte..end_byte)
    }
}

/// Visitor that reads the Nth function call or macro's argument (by index)
struct IndexedLiteralReader {
    target_index: usize,
    current_index: usize,
    tokens: Option<proc_macro2::TokenStream>,
    skip_macros: bool,
}

impl<'ast> syn::visit::Visit<'ast> for IndexedLiteralReader {
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        // Must match the same traversal logic as IndexBuilder
        match node {
            syn::Expr::Call(call) => {
                // Use LAST arg (trailing position)
                if self.tokens.is_none() && self.current_index == self.target_index {
                    if let Some(last_arg) = call.args.last() {
                        self.tokens = Some(quote::quote!(#last_arg));
                    }
                }
                self.current_index += 1;

                self.visit_expr(&call.func);

                let was_skipping = self.skip_macros;
                self.skip_macros = true;
                for arg in &call.args {
                    self.visit_expr(arg);
                }
                self.skip_macros = was_skipping;
                return;
            }
            syn::Expr::MethodCall(method) => {
                // Use RECEIVER
                if self.tokens.is_none() && self.current_index == self.target_index {
                    let receiver = &method.receiver;
                    self.tokens = Some(quote::quote!(#receiver));
                }
                self.current_index += 1;

                self.visit_expr(&method.receiver);

                let was_skipping = self.skip_macros;
                self.skip_macros = true;
                for arg in &method.args {
                    self.visit_expr(arg);
                }
                self.skip_macros = was_skipping;
                return;
            }
            syn::Expr::Macro(mac) => {
                if !self.skip_macros {
                    if self.tokens.is_none() && self.current_index == self.target_index {
                        let tokens = mac.mac.tokens.clone();
                        if !tokens.is_empty() {
                            self.tokens = Some(tokens);
                        }
                    }
                    self.current_index += 1;
                }
                return;
            }
            _ => {}
        }

        syn::visit::visit_expr(self, node);
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

/// Get the stable index for a literal macro at the given position
pub fn get_macro_index(path: &Path, line: u32, column: u32) -> Result<usize, io::Error> {
    let state = get_or_load_file_state(path)?;
    state.get_index(line, column)
}

/// Update a literal macro by its stable index
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

/// Convenience function: update a literal macro at the given position
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

/// Get the current tokens of a literal macro by its stable index
pub fn get_macro_tokens_by_index(
    path: &Path,
    index: usize,
) -> Result<proc_macro2::TokenStream, Box<dyn std::error::Error>> {
    let state = get_or_load_file_state(path)?;
    state.get_macro_tokens(index).map_err(|e| e.into())
}

/// Clear all cached file states (for testing only)
///
/// This is useful in tests when files are manually modified outside
/// the normal runtime update flow. In production, the cache is managed
/// automatically through the update_macro_by_index flow.
///
/// **WARNING:** This is for testing purposes only. Do not use in production code.
#[doc(hidden)]
pub fn clear_file_state_cache() {
    // Clear the global FILE_STATES cache
    let mut states = FILE_STATES.write();
    states.clear();
    drop(states); // Release lock before accessing thread-local

    // Clear the thread-local parsed AST cache
    CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
}

/// Replace an entire function call expression with new tokens.
///
/// Unlike `update_macro_by_index` which replaces only the call's argument,
/// this replaces the entire `func(arg)` expression with the replacement tokens.
///
/// Used by `replace_me()` to substitute the whole call with the baked value.
pub fn replace_expression(
    path: &Path,
    index: usize,
    replacement: proc_macro2::TokenStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = get_or_load_file_state(path)?;
    state.replace_expression_by_index(index, replacement)?;
    write_to_disk(path)?;
    Ok(())
}
