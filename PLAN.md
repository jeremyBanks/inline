# jeb-literal: Implementation Plan

This document outlines planned features and design decisions for future development.

---

## 1. Default Values for Empty Macros

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

1. **Default values for empty macros** - Simple, high value
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
