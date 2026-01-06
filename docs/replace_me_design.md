# `replace_me` Feature Design

## Overview

`replace_me<T>(value: T) -> T` is a one-shot code generation function that:
1. On first execution: evaluates the expression, writes it to source code, returns the value
2. On subsequent executions: returns a clone of the persisted value (ignoring the new argument)
3. The source file transformation replaces the entire `replace_me(...)` call with the literal value

## Motivation

While `code_cell` provides ongoing mutable persistence, `replace_me` serves a different use case:
- **One-time code generation**: Generate a value once, bake it into source, never need the function again
- **Build-time constants**: Compute expensive values once during development
- **Snapshot testing style**: Capture computed values as source literals

Example transformation:
```rust
// Before (first run)
let uuid = replace_me(Uuid::new_v4());

// After (source file is modified)
let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
// or if we can bake Uuid directly:
let uuid = uuid::Uuid(/* baked representation */);
```

## Detailed Semantics

### Return Type: `T` (not a wrapper)

Unlike `code_cell()` which returns `CodeCell<T>`, `replace_me()` returns `T` directly:
- No ongoing shared state needed after replacement
- Callers get owned values
- Simpler API for the primary use case

### Memory Persistence (Pre-replacement)

Before the source is replaced, the in-memory persistence mechanism is still needed:

```rust
fn generate_config() -> Config {
    replace_me(Config::compute_expensive())
}

// Multiple calls during the same run:
let c1 = generate_config(); // First: computes, stores in memory, writes to disk
let c2 = generate_config(); // Later: returns clone from memory (no recompute)
let c3 = generate_config(); // Later: returns clone from memory (no recompute)
```

Key behaviors:
- **First call**: Evaluates argument, stores in registry, writes to source, returns value
- **Subsequent calls (same run)**: Returns `clone()` of stored value, ignores argument
- **After replacement**: The `replace_me()` call no longer exists in source

### Argument Mismatch Handling

If subsequent calls pass different argument values:
- **Normal mode**: Silently ignore the difference, return stored value
- **Verify mode**: Panic if argument differs from stored value (catches non-determinism bugs)

```rust
// Potentially non-deterministic
replace_me(SystemTime::now())  // Second call would differ - verify mode catches this
```

### Mode Behavior

| Mode | First Call | Subsequent Calls | Source Update |
|------|------------|------------------|---------------|
| Write | Evaluate + store + return | Clone from memory | Yes |
| Memory | Evaluate + store + return | Clone from memory | No |
| Verify | Evaluate + verify match | Verify + clone | N/A |
| Reject | Panic | Panic | N/A |

## Source Transformation

### What Gets Replaced

The entire `replace_me(...)` expression is replaced with the baked literal:

```rust
// Before
let x = replace_me(vec![1, 2, 3]);

// After
let x = <[_]>::into_vec(Box::new([1i32, 2i32, 3i32]));
```

### AST Modification Strategy

Unlike `code_cell` which replaces only the *argument* tokens, `replace_me` replaces the *entire call*:

Current `code_cell` approach:
```rust
// Source:       code_cell(42u32)
// Replacement:  code_cell(100u32)  <- only argument changes
```

New `replace_me` approach:
```rust
// Source:       replace_me(compute())
// Replacement:  baked_value         <- entire call removed
```

This requires different span handling:
- `code_cell`: Find the argument span within the call
- `replace_me`: Find the entire `ExprCall` span

### Span Calculation

Using `#[track_caller]`:
- `Location::caller()` gives us line/column of the *function name* start
- Need to find the matching `ExprCall` and get its *full span* (including closing paren)

```rust
fn find_replace_me_span(file: &Path, line: u32, col: u32) -> (usize, usize) {
    // Parse file, find ExprCall at (line, col)
    // Return (start_byte, end_byte) of entire expression
}
```

### Handling Nested Expressions

```rust
let x = foo(replace_me(bar()));
//          ^------------------^ this span
```

The replacement affects only the `replace_me(...)` portion, not the outer call.

## Implementation Plan

### 1. Core Function

```rust
// src/replace.rs

#[track_caller]
pub fn replace_me<T: Value + Clone + 'static>(value: T) -> T {
    let loc = std::panic::Location::caller();
    replace_me_at(value, loc.file(), loc.line(), loc.column())
}

pub fn replace_me_at<T: Value + Clone + 'static>(
    value: T,
    file: &str,
    line: u32,
    column: u32,
) -> T {
    // 1. Check registry for existing value at this location
    // 2. If exists: return clone (ignore `value` argument)
    // 3. If not: store value, trigger replacement, return value
}
```

### 2. Registry Changes

Need a separate registry or marker for `replace_me` vs `code_cell`:
- `replace_me` entries are "fire-once" - no ongoing mutation expected
- Could share the same storage but with different write behavior

### 3. Source Writer Changes

New function to replace entire expression:

```rust
// src/runtime.rs

pub fn replace_expression(
    file: &Path,
    line: u32,
    column: u32,
    replacement: proc_macro2::TokenStream,
) -> Result<(), Error> {
    // 1. Parse file
    // 2. Find ExprCall at position
    // 3. Get full span of expression
    // 4. Replace source[start..end] with replacement tokens
}
```

### 4. Verify Mode Integration

In verify mode, check that the baked value matches what would be computed:

```rust
fn verify_replace_me<T: Value + PartialEq>(stored: &T, computed: &T) {
    assert!(
        stored == computed,
        "replace_me value mismatch: source has {:?} but computed {:?}",
        stored, computed
    );
}
```

Note: Requires `T: PartialEq` for meaningful verification.

### 5. Dead Import Handling

After replacement, `use code_cell::replace_me` becomes unused. Per user decision:
- **Do not attempt cleanup**
- Dead imports are acceptable
- Users can clean manually or let `rustfmt`/linters handle it

## Edge Cases

### 1. Replacement Already Happened

If the source no longer contains `replace_me(...)`:
- The function isn't called at all (it's been replaced)
- No special handling needed

### 2. File Not Found / Parse Error

Same handling as `code_cell`:
- In write mode: panic or error
- In memory mode: continue in-memory only

### 3. Multiple Calls on Same Line

Position-based matching handles this correctly:
```rust
let (a, b) = (replace_me(1), replace_me(2));
//            ^col=12         ^col=28
```

Each has distinct column, works as expected.

### 4. Generic Types

```rust
replace_me::<Vec<u32>>(vec![1, 2, 3])
```

The turbofish is part of the expression and gets replaced entirely.

### 5. Macro Invocations in Argument

```rust
replace_me(format!("hello {}", name))
```

Works fine - the argument is evaluated, result is baked.

## Testing Strategy

1. **Basic replacement**: Verify source file is modified correctly
2. **Memory persistence**: Multiple calls return same value
3. **Clone semantics**: Returned values are independent
4. **Mode behavior**: Test all four modes
5. **Verify mode**: Catches non-determinism
6. **Complex types**: Nested structures, generics, custom Bake impls

## API Summary

```rust
// Primary function
#[track_caller]
pub fn replace_me<T: Value + Clone + 'static>(value: T) -> T;

// For testing with explicit location
pub fn replace_me_at<T: Value + Clone + 'static>(
    value: T,
    file: &str,
    line: u32,
    column: u32,
) -> T;
```

---

## Future Work: Extension Trait Methods

After `replace_me` is implemented, we plan to add extension trait methods:

```rust
pub trait ReplaceExt: Value + Clone + Sized + 'static {
    #[track_caller]
    fn replace_me(self) -> Self;

    #[track_caller]
    fn code_cell(self) -> CodeCell<Self>;
}

impl<T: Value + Clone + 'static> ReplaceExt for T {
    fn replace_me(self) -> Self {
        crate::replace::replace_me(self)
    }

    fn code_cell(self) -> CodeCell<Self> {
        crate::code_cell(self)
    }
}
```

This enables fluent syntax:
```rust
use code_cell::ReplaceExt;

let uuid = Uuid::new_v4().replace_me();
let counter = 0u32.code_cell();
```

This is a separate project to tackle after the core `replace_me` function is complete.
