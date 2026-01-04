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

## 3. Serde Compatibility (Future Goal)

**Current**: Only types implementing `databake::Bake`
**Goal**: Support any type with `Serialize + Deserialize`

### Challenge
databake produces **Rust code tokens**, not serialized data:
```rust
// databake::Bake
vec![1u32, 2u32, 3u32]  // Produces: vec![1u32, 2u32, 3u32]

// serde::Serialize
vec![1u32, 2u32, 3u32]  // Produces: [1, 2, 3] (JSON/bincode/etc)
```

### Approach
Need a layer that converts serde output → Rust tokens:
- Use `ron` (Rusty Object Notation) or similar
- Parse serialized form back to TokenStream
- Insert into source as Rust code

### Benefits
- Much wider type support (any serde type)
- Community ecosystem (serde is ubiquitous)
- Custom types don't need databake impls

### Noted in README
Already documented as future goal:
> **Future Goal**: Add serde compatibility to support any type implementing `Serialize + Deserialize`, expanding beyond databake's current type coverage.

---

## 4. Line Number Stability (Current Limitation)

### Problem
Registry keys use `(file, line, column, TypeId)`:
```rust
let counter = literal!(0u32);  // Registered at line 42
// Add lines above...
let counter = literal!(0u32);  // Now at line 50, different registry entry!
```

### Solutions

**Short term**: File-backed literals solve this
- External file path is stable
- Survives refactoring

**Long term**: Semantic anchoring
- Use AST structure instead of line numbers
- Example: "3rd literal! in function `test_user`"
- More complex to implement

---

## 5. Concurrent Modification Detection

**Current**: Detects when external process modifies source file
**Behavior**: Panics to prevent data loss

### Working Well
- File hash comparison
- Clear panic message
- Prevents silent corruption

### Future Enhancement
Could offer "merge" mode:
- Parse both versions
- Attempt automatic merge
- Fall back to conflict markers

Not high priority - current panic-on-conflict is safe.

---

## 6. Tooling Integration

### cargo-literal (Future)
CLI tool for managing snapshots:

```bash
# Review all changed snapshots
cargo literal diff

# Accept all snapshot updates
cargo literal accept

# Reject and restore original
cargo literal reject

# Clean orphaned snapshot files
cargo literal clean

# Interactive review (like git add -p)
cargo literal review
```

### IDE Integration
- Inline diff preview for changed literals
- "Accept snapshot" code action
- Snapshot file navigation

---

## Implementation Priority

1. **Default values for empty macros** - Simple, high value
2. **DerefMut + write-on-drop** - Better ergonomics, natural Rust patterns
3. **cargo-literal tooling** - Developer experience
4. **Serde compatibility** - Ecosystem integration

---

## Design Principles

1. **Explicit over implicit** - `.set()` is clear
2. **Fail fast** - Panic on conflicts, don't silently corrupt
3. **No magic** - Behavior should be predictable
4. **Lazy evaluation** - Only do work when needed
5. **Minimal trait bounds** - Avoid Clone, PartialEq where possible
6. **Compile-time correctness** - Leverage type system

---

## Open Questions

1. Should `literal_file!()` panic or return Result on first run?
2. Path generation strategy for auto-generated snapshot paths?
3. How to handle snapshot file encoding (UTF-8, escaping)?
4. Should we support custom `Bake` implementations for format control?
5. Multi-file snapshot support (split large values across files)?

---

*This plan is a living document. Update as design evolves.*
