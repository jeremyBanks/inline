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

## 2. File-Backed Literals via `include!()`

**Goal**: Support external snapshot files while maintaining identical runtime semantics to inline literals.

### Motivation
- **Stable identifiers**: Path-based keys instead of (file, line, column)
- **Better diffs**: Changes show in external file, not source code
- **Large values**: Keep big snapshots out of source files
- **Organization**: Group related snapshots in directory structure

### API Design

```rust
// Option A: Auto-generated path
let user = literal_file!(User::default());
// After first write, source becomes:
// let user = literal!(include!("snapshots/tests_rs/L42_C5.snap"));

// Option B: Custom path
let user = literal_file!("snapshots/user.snap", User::default());
// After first write:
// let user = literal!(include!("snapshots/user.snap"));

// Then both work identically:
*user  // reads value
user.set(updated_user);  // updates external file
```

### Macro Expansion Order
Rust expands macros **outside-in**:
```rust
literal!(include!("snapshots/counter.snap"))
```
1. `include!()` expands to file contents: `42u32`
2. `literal!(42u32)` expands to registry code

At compile time, behaves identically to `literal!(42u32)`.

### Implementation Phases

**Phase 1: Detection**
- Parse tokens inside `literal!(...)` macro call
- Detect pattern: `include!("path")` or `include!(concat!(...))`
- Extract external file path from token stream

```rust
// In LiteralInner::set()
let tokens = /* parse from source */;
if is_include_macro(&tokens) {
    let path = extract_include_path(&tokens);
    update_external_file(&path, new_value)?;
} else {
    update_source_file(new_value)?;
}
```

**Phase 2: literal_file!() macro**
- New macro that generates file path
- Writes initial value to external file
- Rewrites source to use `literal!(include!(...))`
- Panics with "Recompile required" message

**Phase 3: Registry key stability**
```rust
enum RegistryKey {
    Inline {
        file: PathBuf,
        line: u32,
        column: u32,
        type_id: TypeId,
    },
    External {
        snapshot_path: PathBuf,
        type_id: TypeId,
    },
}
```

External keys are **stable across refactoring** - don't break when lines shift.

### Example Workflow

**Initial code:**
```rust
#[test]
fn test_user_creation() {
    let expected = literal_file!("snapshots/user.snap", User {
        id: 1,
        name: "Alice",
    });

    let actual = create_user("Alice");
    assert_eq!(actual, *expected);
}
```

**First run:**
1. Creates `snapshots/user.snap` with serialized User
2. Rewrites source to: `literal!(include!("snapshots/user.snap"))`
3. Exits with "Recompile required"

**After recompile:**
```rust
#[test]
fn test_user_creation() {
    let expected = literal!(include!("snapshots/user.snap"));
    // Rest unchanged
}
```

**On test failure:**
```bash
LITERAL_MODE=write cargo test  # Updates external file
git diff snapshots/user.snap   # Review changes
```

### Challenges
- **Two-phase setup**: Requires recompile after first run
- **Mental model**: Two flavors of literals to understand
- **Orphaned files**: Deleted source lines leave external files behind (needs cleanup tooling)

### Future: Snapshot Directory Management
```rust
// In Cargo.toml or .literal-config
[literal]
snapshot_dir = "snapshots"
cleanup_orphans = true  // Remove unreferenced snapshot files
```

---

## 3. DerefMut + Write-on-Drop (Not Recommended)

**Goal**: Enable mutation through `&mut T` instead of explicit `.set()`.

### Proposed API
```rust
let mut counter = literal!(0u32);
*counter += 1;  // Mutate through DerefMut
// On drop: detect change and write to source
```

### Why Current Design is Better

**Current design:**
- ✅ No Clone requirement (moves values)
- ✅ Explicit mutation points (clear intent)
- ✅ Lazy evaluation (only bakes on `.set()`)
- ✅ Token-based comparison (semantic equality)
- ✅ No overhead for read-only uses

**DerefMut + write-on-drop would require:**
- ❌ Clone trait bound OR immediate baking overhead
- ❌ Baking on every creation and drop (even unchanged values)
- ❌ Less explicit mutation points
- ⚠️ Tolerable: Immediate baking on creation (acceptable if needed)

### Detection Challenge
Once you hand out `&mut T`, there's **no way to know if it was actually mutated** without comparing old vs new values.

**Options:**
1. Store original value (requires Clone) - **rejected**
2. Bake on creation + bake on drop, compare tokens - **tolerable but expensive**
3. Use dirty flag with Cell - **doesn't work**, can't track field-level mutations

### Decision
**Not implementing** for now. Current explicit `.set()` API is better for the snapshot testing use case. Could reconsider if strong user demand emerges.

---

## 4. Serde Compatibility (Future Goal)

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

## 5. Line Number Stability (Current Limitation)

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

## 6. Concurrent Modification Detection

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

## 7. Tooling Integration

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
2. **File-backed literals (Phase 1: Detection)** - Foundation
3. **File-backed literals (Phase 2: literal_file! macro)** - Usability
4. **Snapshot directory management** - Maintenance
5. **cargo-literal tooling** - Developer experience
6. **Serde compatibility** - Ecosystem integration

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
