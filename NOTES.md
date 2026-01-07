# Implementation Notes: Removing Macros

## Summary: SUCCESS ✅

The macros **can** be removed and replaced with `#[track_caller]` functions.

## Key Insight

`std::panic::Location::caller()` provides the same information as `file!()`, `line!()`, `column!()`:
- `location.file()` → `&'static str` (same as `file!()`)
- `location.line()` → `u32` (same as `line!()`)
- `location.column()` → `u32` (same as `column!()`)

The column from `#[track_caller]` points to the function call site, which is what we need
for AST scanning.

## Changes Made

### 1. `src/inline.rs`
- Removed `macro_rules! literal`
- Added `#[track_caller] pub fn literal<T: Value + 'static>(value: T) -> Literal<T>`
- Removed `literal_default` (was only needed for empty macro case which can't happen now)

### 2. `src/runtime.rs`
- Changed AST scanning from `ExprMacro`/`StmtMacro` to `ExprCall`
- Updated `build_index_map()` - now uses `visit_expr` and looks for `Expr::Call`
- Updated `find_literal_arg_span_static()` - finds function argument span instead of macro tokens
- Updated `IndexedLiteralReader` - reads function call arguments
- Renamed `macro_index` to `literal_index` throughout

### 3. Added dependency
- Added `quote = "1"` to Cargo.toml (for token quoting in reader)

### 4. Updated all tests
- Changed helper functions from `visit_expr_macro` to `visit_expr`
- Updated source code strings from `literal!(x)` to `literal(x)`
- Removed default_values_serial_test.rs (feature no longer exists)

## What Worked

- `#[track_caller]` provides accurate call-site location
- AST scanning for function calls (`ExprCall`) works as well as for macros
- Column matching works correctly for function calls
- All core tests pass

## Position-Based Matching (Improved)

Initially, the AST scanning code filtered by function name (`segment.ident == "literal"`).
This was fragile because:
- Name collisions with other functions named `literal`
- Aliases (`use literal as foo; foo(42)`)
- Qualified paths (`crate::literal()`, `inline::literal()`)

**Solution:** Match by exact position only, not by name.

The key insight: `#[track_caller]` gives us the exact (line, column) of the call site.
Whatever function call is at that position IS our call - guaranteed because we called it.
We don't need to filter by name at all.

Changes in `src/runtime.rs`:
- `build_index_map()` - indexes ALL `ExprCall` by position, not just ones named "literal"
- `find_literal_arg_span_static()` - finds the Nth call expression, not the Nth "literal" call
- `IndexedLiteralReader` - reads the Nth call expression's argument

This makes the library work with any function name, aliases, or qualified paths.

## API Change

| Before (Macro) | After (Function) |
|----------------|------------------|
| `literal!(42u32)` | `literal(42u32)` |
| `literal!()` | `literal_default::<T>()` |

## Benefits

1. **Simpler** - functions are easier to understand than macros
2. **Better IDE support** - autocomplete, go-to-definition work properly
3. **Clearer errors** - function type errors are more readable
4. **No hygiene concerns** - no macro hygiene edge cases
