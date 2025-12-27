# Technical Challenges and Solutions

## Challenge 1: Accurate Location Tracking

### The Problem
We need to find the exact `litter!()` macro invocation in the AST using only `(file, line, column)` from `file!()`, `line!()`, `column!()` macros.

### Why It's Hard
- `file!()` etc. give us the location of the macro invocation
- `syn::parse_file()` parses source text into an AST
- Spans in the parsed AST may or may not preserve original line/column info

### Solution
Use `proc_macro2::Span` which preserves source locations:

```rust
let ast = syn::parse_file(source)?;

// When visiting:
impl VisitMut for MacroReplacer {
    fn visit_expr_macro_mut(&mut self, node: &mut syn::ExprMacro) {
        if let Some(ident) = node.mac.path.get_ident() {
            let span = ident.span();
            let start = span.start();

            if start.line == target_line && start.column == target_column {
                // Found it!
            }
        }
    }
}
```

### Fallback Plan
If spans don't work:
1. Manually count lines and columns in source text
2. Use byte offsets instead
3. Add explicit IDs: `litter!(id = "counter", 0)`

### Validation Test
```rust
#[test]
fn verify_span_accuracy() {
    let source = "fn main() {\n    let x = litter!(42);\n}";
    let ast = syn::parse_file(source).unwrap();
    // Find macro and verify span.start() == (2, 13)
}
```

---

## Challenge 2: Shared Mutable AST

### The Problem
Multiple `Litter` instances in the same file need to:
- Share the same parsed AST (to avoid reparsing)
- Mutate it independently (to update different values)
- Synchronize access (to prevent corruption)

### Why It's Hard
- `Arc<FileState>` is shared but immutable
- Can't get `&mut` to the AST when wrapped in Arc
- Need interior mutability without UB

### Solution
Use `RefCell` for interior mutability:

```rust
struct FileState {
    ast: RefCell<syn::File>,  // Interior mutability
    lock: Mutex<()>,           // Serialize access
}

// Usage:
let _guard = state.lock.lock().unwrap();  // Acquire lock
let mut ast = state.ast.borrow_mut();     // Get mutable reference
visitor.visit_file_mut(&mut *ast);        // Modify
// RefCell ensures no other borrows exist
```

### Risk
`RefCell` panics if borrowing rules are violated at runtime. We prevent this by:
1. Always holding the `Mutex` guard while borrowing
2. Only one borrow_mut() at a time per file
3. No holding references across function boundaries

### Alternative
Instead of RefCell, use `parking_lot::RwLock`:

```rust
struct FileState {
    ast: RwLock<syn::File>,
    // No separate lock needed
}

// Usage:
let mut ast = state.ast.write();  // Exclusive lock
visitor.visit_file_mut(&mut *ast);
```

This might be cleaner - consider switching to this.

---

## Challenge 3: databake Output Format

### The Problem
databake generates fully-qualified Rust code:

```rust
let vec = vec![1, 2, 3];
let baked = vec.bake(&CrateEnv::default());
// Produces: alloc::vec::Vec::<i32>::from([1i32, 2i32, 3i32])
```

This is verbose and may not match the original format.

### Why It Might Not Matter
- The code is correct and compiles
- prettyplease will format it nicely
- Self-modifying scripts don't need human-readable format

### If We Need To Fix It
Options:
1. Configure `CrateEnv` to use shorter paths
2. Post-process the TokenStream to simplify paths
3. Implement custom `Bake` impls with prettier output
4. Use rustfmt on the final output

For now: **accept verbose output**, revisit if it's a problem.

---

## Challenge 4: File State Invalidation

### The Problem
The AST in memory becomes stale if:
- File is edited externally (user edits in IDE)
- Another process modifies the file
- File is moved/renamed

### Why It's Hard
We don't have filesystem watchers or inotify integration.

### Solution (Simple)
**Don't cache indefinitely**. Options:

1. **Reload on every update** (simple, but slow)
   ```rust
   fn update_source(&self, new_value: T) {
       let state = FileState::load(&self.file)?;  // Always fresh
       state.replace_macro_value(...);
       state.write_to_disk(...);
   }
   ```

2. **Cache with TTL** (complex)
   - Store timestamp with FileState
   - Reload if > 1 second old
   - Or check file mtime

3. **Never reload** (simple, fast, risky)
   - Load once, never refresh
   - Assume single process owns the file
   - Works for self-modifying scripts

**Recommendation**: Start with option 3 (never reload), since:
- Self-modifying scripts are single-process
- External edits would break line numbers anyway
- Can add reloading later if needed

### For Production
Add a `reload()` method that users can call explicitly:

```rust
impl FileState {
    pub fn reload(&mut self, path: &Path) {
        let source = fs::read_to_string(path).unwrap();
        self.ast = syn::parse_file(&source).unwrap();
    }
}

// Usage:
litter::reload_file(file!());  // Force reload before updates
```

---

## Challenge 5: Macro Invocation Context

### The Problem
`litter!()` can appear in different contexts:

```rust
let x = litter!(42);              // ExprMacro
const Y: u32 = litter!(42);       // ExprMacro in const
static Z: u32 = litter!(42);      // ExprMacro in static
fn f() -> u32 { litter!(42) }     // ExprMacro in return
litter!(println!("hi"));          // StmtMacro
```

Our visitor only handles `ExprMacro` currently.

### Solution
Implement multiple visitor methods:

```rust
impl VisitMut for MacroReplacer {
    fn visit_expr_macro_mut(&mut self, node: &mut syn::ExprMacro) {
        self.try_replace_macro(&mut node.mac);
        visit_mut::visit_expr_macro_mut(self, node);
    }

    fn visit_stmt_macro_mut(&mut self, node: &mut syn::StmtMacro) {
        self.try_replace_macro(&mut node.mac);
        visit_mut::visit_stmt_macro_mut(self, node);
    }

    fn try_replace_macro(&mut self, mac: &mut syn::Macro) {
        if let Some(ident) = mac.path.get_ident() {
            if ident == "litter" {
                let span = ident.span().start();
                if span.line == self.target_line &&
                   span.column == self.target_column {
                    mac.tokens = self.new_tokens.clone();
                    self.found = true;
                }
            }
        }
    }
}
```

This handles both expression and statement contexts.

### Test
```rust
#[test]
fn test_stmt_macro() {
    let source = r#"
fn main() {
    litter!(println!("hello"));
}
"#;
    // Should be able to find and replace
}
```

---

## Challenge 6: Type Erasure in Generic Litter<T>

### The Problem
```rust
let x = litter!(42);  // What is T?
```

Without type annotations, Rust infers `i32`. But we need to know the type to:
- Serialize with databake
- Verify type matches on reload

### Current Non-Issue
For now, this isn't a problem because:
- User specifies the type explicitly: `litter!(42u32)`
- Or Rust infers it from usage: `let x: u32 = litter!(42);`
- databake handles the types that have `Bake` implemented

### Future Issue
If we want to support:
```rust
let x = litter!(MyStruct { ... });
// Later, how do we know it's MyStruct when parsing?
```

We'd need to:
1. Store type info in the macro: `litter!("MyStruct", MyStruct { ... })`
2. Or parse the tokens and infer type from structure
3. Or require type annotations always

**For now**: Not a problem. Revisit if we add custom Bake impls.

---

## Challenge 7: Formatting Stability

### The Problem
After updates, prettyplease formats the code. This might:
- Change indentation
- Reorder imports
- Modify spacing

Users may not want their code reformatted.

### Example
Original:
```rust
fn main() {
    let x=litter!(42);    // weird spacing
}
```

After update:
```rust
fn main() {
    let x = litter!(100); // normalized spacing
}
```

### Solution Options

1. **Accept formatting changes** (simple)
   - Document that litter normalizes code
   - Use prettyplease's default style
   - Consistent, predictable output

2. **Preserve original formatting** (complex)
   - Don't use prettyplease
   - Manually replace just the macro tokens in the source text
   - Requires precise byte offset tracking
   - Fragile if source changes

3. **Hybrid: format only changed expressions** (medium)
   - Replace macro tokens in AST
   - Serialize the AST back to string without full formatting
   - Use quote! to render the modified parts

**Recommendation**: Start with option 1 (accept formatting).

Rationale:
- Self-modifying scripts are often auto-generated
- Consistent formatting is usually desirable
- Much simpler implementation
- Can add format-preserving mode later if needed

---

## Challenge 8: Error Recovery

### The Problem
What if updating fails?

```rust
litter.set(new_value);
// Filesystem full? Parse error? Permission denied?
```

The in-memory value is updated but file write failed.

### Solution
Transactional updates:

```rust
pub fn set(&mut self, new_value: T) -> Result<(), Error> {
    if self.value == new_value {
        return Ok(());
    }

    let old_value = self.value.clone();

    // Try to update file first
    self.update_source(&new_value)?;

    // Only update in-memory value if file write succeeded
    self.value = new_value;

    Ok(())
}
```

Or optimistically update, rollback on failure:

```rust
pub fn set(&mut self, new_value: T) {
    if self.value == new_value {
        return;
    }

    let old_value = self.value.clone();
    self.value = new_value.clone();

    if let Err(e) = self.update_source(&new_value) {
        // Rollback
        self.value = old_value;
        eprintln!("Failed to update source: {}", e);
    }
}
```

**Recommendation**: Optimistic update with rollback.
- Matches current API (no Result return)
- Graceful degradation
- Logs errors but doesn't panic

---

## Summary of Decisions

| Challenge | Solution | Confidence |
|-----------|----------|------------|
| Location tracking | Use proc_macro2 spans | High - standard approach |
| Shared mutable AST | RefCell or RwLock | High - proven pattern |
| databake format | Accept verbose output | Medium - may need tweaking |
| File invalidation | Never reload | Medium - works for our use case |
| Macro contexts | Handle Expr + Stmt | High - covers 99% of cases |
| Type erasure | Explicit annotations | High - not an issue yet |
| Formatting stability | Accept formatting changes | Medium - may need option later |
| Error recovery | Optimistic with rollback | High - safe and simple |

## Next Steps

1. Implement Phase 1 (core infrastructure)
2. Run Test 1 (span preservation) to validate key assumption
3. Iterate based on test results
4. Add missing visitor methods if needed
5. Handle errors gracefully
