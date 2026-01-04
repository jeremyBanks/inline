# Implementation Plan: Public `.value` Field

**Goal**: Enable ergonomic `counter.value = 42` syntax by exposing a public field, eliminating the need for `*` dereference or `.set()` method calls.

## Overview

After investigation, we confirmed that:
- ✅ We already write changes on Drop
- ✅ Drop can detect mutations by comparing `original` vs current value
- ✅ We can expose a public field and Drop will intercept before destruction

The approach: Move the working value out of the guard into a public field on `Literal<T>`, then sync it back to the guard in Drop before writing to file.

## Implementation Strategy

### Current Structure
```rust
pub struct Literal<T: Value + 'static> {
    guard: parking_lot::MutexGuard<'static, LiteralInner<T>>,  // Contains the value
    original: T,  // For mutation detection
}
```

### New Structure
```rust
pub struct Literal<T: Value + 'static> {
    pub value: T,  // ← PUBLIC working copy
    guard: parking_lot::MutexGuard<'static, LiteralInner<T>>,  // For file operations
    original: T,  // For mutation detection
}
```

## Priorities

### Priority #0: Restructure `Literal<T>` with public field
**Status**: Pending

**Changes needed**:
- `src/inline.rs`:
  - Add `pub value: T` field to `Literal<T>`
  - Update `from_guard()` to clone value twice (once for `value`, once for `original`)
  - Update `Deref` impl to return `&self.value` instead of `&self.guard`
  - Update `DerefMut` impl to return `&mut self.value` instead of `&mut self.guard.value`
  - Update `Drop` impl to sync `self.value` back to `self.guard.value` before calling `update_source()`

**Test coverage**: Existing tests should pass (internal change only)

### Priority #1: Deprecate `.set()` method
**Status**: Pending

**Rationale**: With `pub value` field, `.set()` becomes redundant:
- `counter.value = 42` is just as explicit
- `*counter = 42` still works via DerefMut
- Both trigger write-on-drop automatically

**Options**:
1. **Remove entirely** - breaking change but cleaner API
2. **Deprecate** - mark with `#[deprecated]` for gradual migration
3. **Keep** - maintain backward compatibility

**Recommendation**: Remove entirely, as this is pre-1.0 (version 0.0.1-dev.1)

**Changes needed**:
- `src/inline.rs`: Remove `pub fn set(&mut self, new_value: T)` method from `Literal<T>` impl

### Priority #2: Add tests for `.value =` syntax
**Status**: Pending

**New test file**: `tests/value_field_serial_test.rs`

**Test cases**:
1. `test_value_field_assignment_writes_on_drop` - Basic `counter.value = 42` works
2. `test_value_field_with_complex_types` - String, Vec, etc.
3. `test_value_field_no_write_if_unchanged` - Assigning same value doesn't write
4. `test_value_field_multiple_mutations` - Multiple assignments before drop
5. `test_all_three_syntaxes_equivalent` - Show `counter.value = `, `*counter = `, and method calls all work

**Verification**: Ensure existing DerefMut tests still pass (backward compatibility)

### Priority #3: Update documentation
**Status**: Pending

**Files to update**:
- `README.md`:
  - Main example: Show `counter.value = 42` as primary syntax
  - Update "Update it" section to show three syntaxes:
    1. `counter.value = 42` (most explicit, no `*`)
    2. `*counter = 42` (DerefMut shorthand)
    3. `counter.push(3)` (auto-deref for methods)
  - Remove references to `.set()` method
  - Update "How It Works" to mention public field

- `src/inline.rs`:
  - Update `Literal<T>` doc comments with new examples
  - Add note about public `value` field
  - Update macro doc comments

## Tradeoffs

### Performance
- **Cost**: One extra `clone()` per literal creation (2 clones instead of 1)
- **Impact**: Negligible for typical use cases (counters, small configs)
- **Benefit**: Most ergonomic API possible

### API Design
- **Removed**: `.set()` method (explicit but verbose)
- **Added**: Public `.value` field (explicit and ergonomic)
- **Kept**: `*counter =` via DerefMut (shorthand for primitives)
- **Result**: Users have natural choices for different situations

### Migration
- Breaking change for any code using `.set()`
- But: Pre-1.0 version (0.0.1-dev.1) allows breaking changes
- Alternative syntax exists: `.value =` is direct replacement

## Success Criteria

1. ✅ All existing tests pass
2. ✅ New tests demonstrate `.value =` syntax works
3. ✅ Documentation shows all three syntaxes clearly
4. ✅ No performance regression (acceptable to have 1 extra clone)
5. ✅ API feels natural and Rust-idiomatic

## Questions to Resolve

None - investigation confirmed this approach is sound.

## Notes

- This is the most ergonomic API possible in Rust without macros
- The `.value` field approach is used by other Rust libraries (e.g., `RefCell::borrow_mut().value`)
- Auto-deref for method calls already works and will continue to work
- The only case requiring syntax choice is direct assignment (primitive values)
