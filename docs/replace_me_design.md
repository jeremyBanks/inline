# `replace()` Feature Design

## Overview

`replace<T>(value: T) -> T` is a one-shot code generation function that:
1. On first execution: evaluates the expression, writes it to source code, returns the value
2. On subsequent executions: returns a clone of the persisted value (ignoring the new argument)
3. The source file transformation replaces the **entire call expression** with the baked value

## Implementation Status

**Status: Implemented** ✅

The `replace()` function and its variants are fully implemented in `src/replace.rs`.

## Motivation

While `cell()` provides ongoing mutable persistence, `replace()` serves a different use case:
- **One-time code generation**: Generate a value once, bake it into source, function call disappears
- **Build-time constants**: Compute expensive values once during development
- **Snapshot testing style**: Capture computed values as source literals

Example transformation:
```rust
// Before (first run)
let uuid = replace(Uuid::new_v4().to_string());

// After (source file is modified)
let uuid = "550e8400-e29b-41d4-a716-446655440000";
```

## API

### Functions

```rust
/// One-shot replacement - evaluates once, replaces entire call with baked value
#[track_caller]
pub fn replace<T: Bake + Clone + PartialEq + 'static>(value: T) -> T;

/// One-shot replacement with default value
#[track_caller]
pub fn replace_default<T: Bake + Clone + PartialEq + Default + 'static>() -> T;
```

### Function Aliases

All aliases behave identically:
- `replace` - Standard name
- `val` - Short alias
- `eval` - "Evaluate once" semantic
- `REPLACE_ME` - Loud, obvious marker

### Macros

```rust
replace!(expression)      // Macro version
replace_default!()        // Macro version with default
```

## Detailed Semantics

### Return Type: `T` (not a wrapper)

Unlike `cell()` which returns `InlineCell<T>`, `replace()` returns `T` directly:
- No ongoing shared state needed after replacement
- Callers get owned values
- Simpler API for the primary use case

### Memory Persistence (Pre-replacement)

Before the source is replaced, the in-memory persistence mechanism is still needed:

```rust
fn generate_config() -> Config {
    replace(Config::compute_expensive())
}

// Multiple calls during the same run:
let c1 = generate_config(); // First: computes, stores in memory, writes to disk
let c2 = generate_config(); // Later: returns clone from memory (no recompute)
let c3 = generate_config(); // Later: returns clone from memory (no recompute)
```

Key behaviors:
- **First call**: Evaluates argument, stores in registry, writes to source, returns value
- **Subsequent calls (same run)**: Returns `clone()` of stored value, ignores argument
- **After replacement**: The `replace()` call no longer exists in source

### Mode Behavior

| Mode | First Call | Subsequent Calls | Source Update |
|------|------------|------------------|---------------|
| Write | Evaluate + store + return | Clone from memory | Yes (entire call replaced) |
| Memory | Evaluate + store + return | Clone from memory | No |
| Verify | Evaluate + verify match | Verify + clone | N/A |
| Reject | Panic | Panic | N/A |

## Source Transformation

### What Gets Replaced

The **entire call expression** is replaced with the baked value:

```rust
// Before
let x = replace(vec![1, 2, 3]);

// After
let x = <[_]>::into_vec(Box::new([1i32, 2i32, 3i32]));
```

### Difference from `cell()`

`cell()` replaces only the value argument, keeping the function call:
```rust
// cell() behavior:
// Before: cell(42u32)
// After:  cell(100u32)  <- only argument changes

// replace() behavior:
// Before: replace(compute())
// After:  "baked_value"  <- entire call removed
```

### Replacement by Call Type

| Call Type | What Gets Replaced |
|-----------|-------------------|
| Function call `replace(x)` | Entire `replace(x)` expression |
| Method call `x.replace_me()` | Entire `x.replace_me()` expression |
| Macro `replace!(x)` | Entire `replace!(x)` expression |

## Implementation Details

### `#[track_caller]` for Location

The function uses `#[track_caller]` to capture the call site:

```rust
#[track_caller]
pub fn replace<T: Bake + Clone + PartialEq + 'static>(value: T) -> T {
    let loc = std::panic::Location::caller();
    replace_impl(value, loc.file(), loc.line(), loc.column())
}
```

### AST Span Calculation

Finding the entire expression span:
- `Location::caller()` gives line/column of the function name start
- AST visitor finds the matching `ExprCall`, `ExprMethodCall`, or `ExprMacro`
- Returns the full span (start to closing paren/bracket)

```rust
// The span finder in runtime.rs handles this:
fn find_call_expression_span_static(
    ast: &syn::File,
    index: usize,
    source: &str,
) -> Result<ByteSpan, String>
```

### Registry Integration

`replace()` shares the same registry infrastructure as `cell()`:
- Same position-based indexing
- Same type-erased storage
- Different write behavior (replace entire expression vs. just argument)

### Dead Import Handling

After replacement, `use inline::replace` becomes unused:
- **Do not attempt cleanup**
- Dead imports are acceptable
- Users can clean manually or let `rustfmt`/linters handle it

## Edge Cases

### 1. Replacement Already Happened

If the source no longer contains `replace(...)`:
- The function isn't called at all (it's been replaced with a literal)
- No special handling needed

### 2. File Not Found / Parse Error

Same handling as `cell()`:
- In write mode: panic or error
- In memory mode: continue in-memory only

### 3. Multiple Calls on Same Line

Position-based matching handles this correctly:
```rust
let (a, b) = (replace(1), replace(2));
//            ^col=12     ^col=25
```

Each has distinct column, works as expected.

### 4. Nested Expressions

```rust
let x = foo(replace(bar()));
//          ^--------------^ this span only
```

The replacement affects only the `replace(...)` portion, not the outer call.

### 5. Generic Types

```rust
replace::<Vec<u32>>(vec![1, 2, 3])
```

The turbofish is part of the expression and gets replaced entirely.

## Testing

Tests are in `tests/macro_syntax_serial_test.rs`:
- `test_replace_macro_basic` - Basic replacement functionality

Additional test coverage needed:
- Multiple calls return same value
- Complex types with Bake implementations
- All four modes
- Expression replacement span calculation

## Future Work

### Extension Trait Method

Could add a method-style API via extension trait:

```rust
use inline::ReplaceExt;

let uuid = Uuid::new_v4().replace_me();
```

This would use the same `#[track_caller]` mechanism.

### Verify Mode Enhancement

In verify mode, could check that computed value matches baked value:
- Catches non-determinism bugs
- Requires `T: PartialEq` (already required)

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-01-05 | Initial design document |
| 0.2.0 | 2026-01-07 | Updated to reflect implementation, `#[track_caller]` approach |
