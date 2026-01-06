# Plan: Remove Macros Using #[track_caller]

## Summary

The crate currently uses `macro_rules! literal` to capture source location via `file!()`, `line!()`, `column!()`. This can be replaced with `#[track_caller]` and `std::panic::Location::caller()`, which provides the same information at runtime.

## Why This Works

### Current Macro Approach
```rust
macro_rules! literal {
    ($value:expr) => {{
        $crate::Literal::from_guard($crate::registry::get_or_create(
            $value,
            file!(),      // Compile-time: caller's file
            line!(),      // Compile-time: caller's line
            column!()     // Compile-time: caller's column
        ).lock())
    }};
}
```

### Replacement with #[track_caller]
```rust
#[track_caller]
pub fn literal<T: Value + 'static>(value: T) -> Literal<T> {
    let loc = std::panic::Location::caller();
    Literal::from_guard(registry::get_or_create(
        value,
        loc.file(),    // Runtime: caller's file (same info!)
        loc.line(),    // Runtime: caller's line
        loc.column()   // Runtime: caller's column
    ).lock())
}
```

`Location::caller()` provides:
- `file()` → `&'static str` (identical to `file!()`)
- `line()` → `u32` (identical to `line!()`)
- `column()` → `u32` (identical to `column!()`)

## Changes Required

### 1. `src/inline.rs` - Replace macro with function

**Remove** (lines 365-378):
```rust
#[macro_export]
macro_rules! literal {
    () => {{ ... }};
    ($value:expr) => {{ ... }};
}
```

**Add**:
```rust
/// Create a self-modifying value that can update its source code.
///
/// # Example
/// ```no_run
/// use jeb_literal::literal;
/// let mut counter = literal(0u32);
/// *counter += 1;
/// ```
#[track_caller]
pub fn literal<T: Value + 'static + Default>(value: T) -> Literal<T> {
    let loc = std::panic::Location::caller();
    Literal::from_guard(crate::registry::get_or_create(
        value,
        loc.file(),
        loc.line(),
        loc.column(),
    ).lock())
}

/// Create a self-modifying value using the type's default.
///
/// # Example
/// ```no_run
/// use jeb_literal::literal_default;
/// let counter: jeb_literal::Literal<u32> = literal_default();
/// ```
#[track_caller]
pub fn literal_default<T: Value + 'static + Default>() -> Literal<T> {
    literal(T::default())
}
```

### 2. `src/runtime.rs` - Change AST scanning from macros to function calls

**Current** (`build_index_map`, lines 232-286): Looks for `ExprMacro`/`StmtMacro` with path ending in `literal`.

**Change to**: Look for `ExprCall` with path ending in `literal`.

```rust
fn build_index_map(ast: &syn::File) -> HashMap<(u32, u32), usize> {
    use syn::visit::Visit;

    struct IndexBuilder {
        map: HashMap<(u32, u32), usize>,
        current_index: usize,
    }

    impl<'ast> Visit<'ast> for IndexBuilder {
        fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
            // Check if this is a call to `literal` or `literal_default`
            let is_literal_call = match &*node.func {
                syn::Expr::Path(path) => {
                    if let Some(segment) = path.path.segments.last() {
                        segment.ident == "literal" || segment.ident == "literal_default"
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if is_literal_call {
                let span = match &*node.func {
                    syn::Expr::Path(path) => path.path.segments.last().unwrap().ident.span(),
                    _ => unreachable!(),
                };
                let start = span.start();
                let pos = (start.line as u32, start.column as u32);
                self.map.insert(pos, self.current_index);
                self.current_index += 1;
            }

            syn::visit::visit_expr_call(self, node);
        }
    }

    let mut builder = IndexBuilder {
        map: HashMap::new(),
        current_index: 0,
    };
    builder.visit_file(ast);
    builder.map
}
```

### 3. `src/runtime.rs` - Update span finding for function call arguments

**Current** (`find_macro_value_span_static`): Finds the span of `mac.tokens` (the macro's token contents).

**Change to**: Find the span of the first argument in `ExprCall`:
- For `literal(42u32)` → span of `42u32`
- For `literal_default()` → insert position inside `()`

### 4. `src/runtime.rs` - Update token reading

**Current** (`IndexedMacroReader`): Reads `mac.tokens`.

**Change to**: Read the first argument's tokens from `ExprCall.args`.

### 5. Update all usages

| Before | After |
|--------|-------|
| `literal!(42u32)` | `literal(42u32)` |
| `literal!()` | `literal_default()` |
| `let x: Literal<T> = literal!()` | `let x: Literal<T> = literal_default()` |

### 6. Update `src/lib.rs` exports

Export the functions instead of the macro:
```rust
pub use inline::{literal, literal_default, Literal, LiteralPrivate};
```

## Files to Modify

1. **`src/inline.rs`** - Replace macro with functions
2. **`src/runtime.rs`** - Change AST visitors from macro to function call handling
3. **`src/lib.rs`** - Update exports
4. **`examples/counter.rs`** - Update usage
5. **`tests/*.rs`** - Update all test usages

## Benefits

1. **Simpler API** - Functions are more intuitive than macros
2. **Better IDE support** - Function calls have better autocomplete/documentation
3. **Clearer error messages** - Rust function errors are clearer than macro errors
4. **No hygiene concerns** - No macro hygiene edge cases
5. **Easier to debug** - Can step through function code

## Edge Cases

### Multiple calls on same line
Both approaches work: the column number distinguishes them.
```rust
let (a, b) = (literal(1), literal(2));  // Different columns
```

### Nested calls
`#[track_caller]` propagates through nested function calls with the attribute.

### Inlined functions
If `literal()` gets inlined by the optimizer, `#[track_caller]` still works - it captures the source location at the call site before inlining.

## Testing Strategy

1. Run existing tests after changes
2. Verify position_stability tests still pass (most important)
3. Verify write/verify modes work correctly
4. Test multiple calls on same line
