# Litter Implementation Plan

## Overview
Implement self-modifying literals that use databake for serialization and maintain a stable in-memory AST per source file.

## Architecture Decisions

### 1. Global State Structure
```rust
static FILE_STATES: Lazy<Mutex<HashMap<PathBuf, Arc<FileState>>>> = ...;

struct FileState {
    ast: syn::File,
    lock: Mutex<()>,  // Serialize updates to this file
}
```

**Rationale**:
- One AST per file, shared by all litter instances in that file
- File-level locking prevents concurrent modification races
- Arc allows cloning the reference while keeping single instance

### 2. Location-Based Identification
- Each `Litter` stores `(file, line, column)` from macro expansion
- Locations are stable because we only modify macro token contents
- No need for unique IDs or complex indexing

### 3. Immediate Writes
- Updates write to disk immediately after modifying AST
- Simple, predictable behavior
- No flush() needed, no lost updates on crash

### 4. databake Integration
- Use existing `Bake` implementations (primitives, std types)
- Don't implement custom Bake traits (for now)
- Focus on infrastructure, not type support

## Implementation Phases

### Phase 1: Core Infrastructure

#### 1.1 Update Dependencies
```toml
[dependencies]
databake = "0.1"
syn = { version = "2", features = ["full", "visit-mut", "extra-traits"] }
proc-macro2 = "1"
quote = "1"
once_cell = "1"
prettyplease = "0.2"
```

#### 1.2 Implement `runtime.rs`
```rust
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct FileState {
    ast: syn::File,
    lock: Mutex<()>,
}

static FILE_STATES: Lazy<Mutex<HashMap<PathBuf, Arc<FileState>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

impl FileState {
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let source = std::fs::read_to_string(path)?;
        let ast = syn::parse_file(&source)
            .map_err(|e| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Parse error: {}", e)
            ))?;

        Ok(FileState {
            ast,
            lock: Mutex::new(()),
        })
    }

    pub fn replace_macro_value(
        &mut self,
        line: u32,
        column: u32,
        new_tokens: proc_macro2::TokenStream,
    ) -> Result<(), String> {
        // TODO: Implement visitor
        todo!()
    }

    pub fn write_to_disk(&self, path: &Path) -> Result<(), std::io::Error> {
        let formatted = prettyplease::unparse(&self.ast);
        std::fs::write(path, formatted)?;
        Ok(())
    }
}
```

#### 1.3 Implement `litter.rs` Core Type
```rust
use databake::Bake;
use std::path::PathBuf;

pub struct Litter<T: Bake + PartialEq + Clone> {
    value: T,
    file: PathBuf,
    line: u32,
    column: u32,
}

impl<T: Bake + PartialEq + Clone> Litter<T> {
    #[doc(hidden)]
    pub fn __new(value: T, file: &str, line: u32, column: u32) -> Self {
        Litter {
            value,
            file: PathBuf::from(file),
            line,
            column,
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, new_value: T) {
        if self.value == new_value {
            return;
        }

        self.value = new_value.clone();

        if let Err(e) = self.update_source(new_value) {
            eprintln!("Warning: Failed to update source file: {}", e);
        }
    }

    fn update_source(&self, new_value: T) -> Result<(), Box<dyn std::error::Error>> {
        // Bake to tokens
        let env = databake::CrateEnv::default();
        let baked = new_value.bake(&env);

        // Get or load file state
        let mut states = crate::runtime::FILE_STATES.lock().unwrap();
        let state = states.entry(self.file.clone())
            .or_insert_with(|| {
                Arc::new(crate::runtime::FileState::load(&self.file)
                    .expect("Failed to load source file"))
            })
            .clone();
        drop(states);

        // Lock this file and update
        let _guard = state.lock.lock().unwrap();

        let mut state_mut = Arc::try_unwrap(state)
            .expect("Multiple references to FileState");

        state_mut.replace_macro_value(self.line, self.column, baked)?;
        state_mut.write_to_disk(&self.file)?;

        Ok(())
    }
}
```

Wait, there's an issue with Arc::try_unwrap - we can't get mutable access easily. Need to use interior mutability:

```rust
struct FileState {
    ast: RefCell<syn::File>,  // Interior mutability
    lock: Mutex<()>,
}
```

#### 1.4 Implement Macro
```rust
#[macro_export]
macro_rules! litter {
    ($value:expr) => {{
        $crate::Litter::__new(
            $value,
            file!(),
            line!(),
            column!(),
        )
    }};
}
```

### Phase 2: AST Visitor Implementation

#### 2.1 Implement MacroReplacer in `runtime.rs`
```rust
use syn::visit_mut::{self, VisitMut};
use proc_macro2::TokenStream;

struct MacroReplacer {
    target_line: usize,
    target_column: usize,
    new_tokens: TokenStream,
    found: bool,
}

impl VisitMut for MacroReplacer {
    fn visit_expr_macro_mut(&mut self, node: &mut syn::ExprMacro) {
        if self.found {
            return;
        }

        // Check if this is the litter macro we're looking for
        if let Some(ident) = node.mac.path.get_ident() {
            if ident == "litter" {
                let span = ident.span();
                let start = span.start();

                if start.line == self.target_line &&
                   start.column == self.target_column {
                    // Found it! Replace tokens
                    node.mac.tokens = self.new_tokens.clone();
                    self.found = true;
                    return;
                }
            }
        }

        // Continue traversing
        visit_mut::visit_expr_macro_mut(self, node);
    }
}

impl FileState {
    pub fn replace_macro_value(
        &mut self,
        line: u32,
        column: u32,
        new_tokens: TokenStream,
    ) -> Result<(), String> {
        let mut replacer = MacroReplacer {
            target_line: line as usize,
            target_column: column as usize,
            new_tokens,
            found: false,
        };

        replacer.visit_file_mut(&mut self.ast);

        if !replacer.found {
            return Err(format!(
                "Could not find litter! macro at line {}, column {}",
                line, column
            ));
        }

        Ok(())
    }
}
```

### Phase 3: Testing Strategy

#### 3.1 Test File Setup
Create `tests/integration_tests.rs`:

```rust
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_file(content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, content).unwrap();
    (dir, file_path)
}

#[test]
fn test_simple_update() {
    let (_dir, path) = setup_test_file(r#"
        use litter::litter;

        fn main() {
            let x = litter!(42);
        }
    "#);

    // TODO: Create Litter instance, update, verify file changed
}
```

#### 3.2 Test Cases
1. **Single value update**: Update one litter, verify AST changes
2. **Multiple values**: Two litter in same file, update both
3. **Type preservation**: Ensure baked type matches original
4. **Concurrent updates**: Spawn threads updating different litters
5. **Error handling**: Invalid file, parse errors, etc.

#### 3.3 Verification Strategy
```rust
fn verify_file_contains(path: &Path, expected: &str) {
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains(expected),
        "File does not contain: {}\nActual content:\n{}",
        expected, content);
}
```

### Phase 4: Issues to Resolve

#### 4.1 Span Information
**Problem**: When `syn::parse_file` parses from a string, does it preserve accurate line/column info?

**Test**:
```rust
let source = r#"
fn main() {
    let x = litter!(42);
}
"#;
let ast = syn::parse_file(source).unwrap();
// Check if macro can be found at line 3, col 13
```

**Solution if spans are relative**:
- Keep line offset when parsing
- Or use syn's SourceFile APIs

#### 4.2 Interior Mutability
**Problem**: Arc<FileState> needs mutability but is shared

**Solution**: Use RefCell inside Arc:
```rust
struct FileState {
    ast: RefCell<syn::File>,
    lock: Mutex<()>,
}

// When updating:
let mut ast_ref = state.ast.borrow_mut();
replacer.visit_file_mut(&mut *ast_ref);
```

#### 4.3 databake Import Paths
**Problem**: Baked code might need imports like `alloc::borrow::Cow`

**Solution**: Either:
- Accept that generated code has full paths
- Or implement import management (complex)
- For now: full paths are fine

### Phase 5: Minimal Working Example

Target usage:
```rust
use litter::litter;

fn main() {
    let mut counter = litter!(0u32);

    println!("Run #{}", counter.get() + 1);

    counter.set(counter.get() + 1);
}
```

After first run, source becomes:
```rust
let mut counter = litter!(1u32);
```

## Implementation Order

1. ✅ Update Cargo.toml
2. ✅ Implement FileState with RefCell
3. ✅ Implement Litter<T> struct
4. ✅ Implement litter! macro
5. ✅ Implement MacroReplacer visitor
6. ✅ Write span preservation test
7. ✅ Write simple update test
8. ✅ Fix any span/location issues
9. ✅ Test with databake-supported types
10. ✅ Add error handling
11. ✅ Document API

## Open Questions

1. **Span accuracy**: Do we get accurate line/column from syn::parse_file?
2. **Macro contexts**: Does litter! work in all positions (expr, stmt, const, etc.)?
3. **Type inference**: Can we avoid explicit type annotations like `litter!(42u32)`?
4. **Performance**: Is parsing/formatting on every update acceptable?

## Success Criteria

- [ ] Can update a single litter value
- [ ] Updates persist to source file
- [ ] Multiple litters in same file work
- [ ] Concurrent updates don't corrupt file
- [ ] Works with databake-supported types (primitives, tuples, vecs)
- [ ] Errors are user-friendly
