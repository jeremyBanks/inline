# jeb-literal: Implementation Plan

This document outlines planned features and design decisions for future development.

---

## 0. Registry Key Stability (CRITICAL FIX) ✓

**Status**: ✅ **IMPLEMENTED** - Registry now uses index-based keys, values persist across line insertions.

**Problem**: Current registry uses `(file, line, column, TypeId)` as keys. When lines shift above a `literal!()` call, the key changes, creating a NEW registry entry and **losing the stored value**.

### Current Broken Behavior
```rust
// First run
let counter = literal!(0u32);  // line 42 → key: (file, 42, col, TypeId)
counter.set(5);                 // Saves 5 to registry

// Insert 10 lines above the literal...

// Second run
let counter = literal!(0u32);  // line 52 → NEW key: (file, 52, col, TypeId)
// Starts at 0 again! Lost value 5!
```

### Solution: Index-Based Keys

Use `(file, index, TypeId)` where `index` is the position of the literal in the file ("Nth `literal!()` macro"). This is **stable across line insertions**.

```rust
// Registry key type
type RegistryKey = (PathBuf, usize, TypeId);  // Changed from (PathBuf, u32, u32, TypeId)

// On first access at (file, line, column):
// 1. Call get_macro_index(file, line, column) → finds index (Nth literal in file)
// 2. Use (file, index, TypeId) as registry key
// 3. Store index in LiteralInner.macro_index

// On subsequent accesses (even if line changed):
// 1. Resolve new (line, column) → same index
// 2. Lookup by (file, index, TypeId) → finds same registry entry ✓
```

### Implementation Notes

- The infrastructure already exists: `get_macro_index()` function and `macro_index` field in `LiteralInner`
- Just need to use index for registry keys instead of line/column
- Modify `registry::get_or_create()` to resolve index on first access

### Testing Requirements

**CRITICAL**: Add tests to verify line-shift stability:

1. **Test: Value persists across line insertions**
   ```rust
   // Set a literal to value X
   // Insert lines above it in the source
   // Verify it still has value X (not reset to initial)
   ```

2. **Test: Multiple literals maintain distinct identities**
   ```rust
   // Create literals A and B
   // Insert lines between them
   // Verify each keeps its own value
   ```

3. **Test: Index resolution is consistent**
   ```rust
   // Verify same literal resolves to same index
   // even when accessed at different line numbers
   ```

---

## 1. Default Values for Empty Macros ✓

**Status**: ✅ **IMPLEMENTED** - `literal!()` with no arguments now uses `Default::default()`.

**Goal**: Allow `literal!()` with no argument to use `Default::default()`.

### Current Behavior
```rust
let counter = literal!(0u32);  // Must provide initial value
```

### Proposed Behavior
```rust
let counter = literal!();  // Uses Default::default() - type inferred
let counter: Literal<u32> = literal!();  // Explicit type annotation
```

For file-backed literals:
```rust
// Empty external file uses Default::default()
let value = literal!(include!("snapshots/empty.snap"));  // File is empty
// Expands to: Default::default()
```

### Implementation
- Macro checks if body is empty
- Emits `::std::default::Default::default()` instead of the expression
- **No Default trait bound** - let it fail naturally with clear error if type doesn't implement Default
- Works for both inline and file-backed variants

### Benefits
- Cleaner snapshot initialization
- Natural Rust idiom for default values
- Less verbose for common case (numeric counters, empty collections)

---

## 2. DerefMut + Write-on-Drop

**Goal**: Enable mutation through `&mut T` with automatic write-on-drop for better ergonomics.

### Proposed API
```rust
let mut counter = literal!(0u32);
*counter += 1;  // Mutate through DerefMut
// On drop: detects change and writes to source automatically
```

### Benefits
- **Better ergonomics**: Natural Rust mutation syntax
- **Less boilerplate**: No explicit `.set()` calls needed
- **Familiar**: Matches normal variable semantics

### Implementation Options

**Option 1: Clone Bound (Preferred)**
```rust
pub trait Value: Bake + Clone {}

impl<T: Value + 'static> Literal<T> {
    // Store clone of original on creation
    original: T,
}

impl<T: Value + 'static> Drop for Literal<T> {
    fn drop(&mut self) {
        // Compare current value to original
        if self.value != self.original {
            // Bake and write to source
        }
    }
}
```

**Pros:**
- Only bakes on actual change
- Clean implementation
- Most types already implement Clone

**Cons:**
- Adds Clone trait bound
- Storage overhead (keeps original)

**Option 2: Immediate Baking (Alternative)**
```rust
pub struct Literal<T: Value + 'static> {
    guard: MutexGuard<'static, LiteralInner<T>>,
    original_tokens: String,  // Baked on creation
}

impl<T: Value + 'static> Drop for Literal<T> {
    fn drop(&mut self) {
        let new_tokens = self.guard.value.bake(&env).to_string();
        if new_tokens != self.original_tokens {
            // Write to source
        }
    }
}
```

**Pros:**
- No Clone bound
- Token comparison (semantic equality)

**Cons:**
- Baking overhead on every creation (even for read-only uses)
- More expensive for large values

### Decision
**Prefer Clone bound** for better performance. If Clone proves problematic for important types, fall back to immediate baking.

### Migration Path
Keep `.set()` method for explicit updates. DerefMut is additive - doesn't break existing code.

---

## Implementation Priority

0. ✅ **Registry key stability** - CRITICAL: Fix value loss when lines shift
1. ✅ **Default values for empty macros** - Simple, high value
2. **DerefMut + write-on-drop** - Better ergonomics, natural Rust patterns

---

## Design Principles

1. **Explicit over implicit** - `.set()` stays available for explicit updates
2. **Fail fast** - Panic on conflicts, don't silently corrupt
3. **No magic** - Behavior should be predictable
4. **Lazy evaluation** - Only do work when needed
5. **Minimal trait bounds** - Only add bounds when necessary (Clone for DerefMut is acceptable)
6. **Compile-time correctness** - Leverage type system

---

*This plan is a living document. Update as design evolves.*
