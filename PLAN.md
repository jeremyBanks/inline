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
// User writes this, and it stays this way (no source rewriting)
let user = literal_in_path!("snapshots/user.snap");

// Macro internally expands to something like:
let user = {
    const PATH: &str = "snapshots/user.snap";
    // Include file at compile time (or use Default::default() if empty)
    let value = include!(PATH);  // File contains: User { ... } or empty → Default::default()
    Literal::from_guard(registry::get_or_create_external(value, PATH).lock())
};

// Usage is identical to inline literals:
*user  // reads value
user.set(updated_user);  // updates external file (NOT source code)
```

**Key insight**: The value comes FROM the path. No default parameter needed.

### Macro Implementation

```rust
#[macro_export]
macro_rules! literal_in_path {
    ($path:literal) => {{
        const PATH: &'static str = $path;
        // Include the snapshot file at compile time
        // If file is empty or contains only whitespace, defaults to Default::default()
        let value = include!(PATH);
        $crate::Literal::from_guard(
            $crate::registry::get_or_create_external(value, PATH, file!(), line!(), column!())
                .lock()
        )
    }};
}
```

### Macro Expansion Order
When the compiler sees:
```rust
literal_in_path!("snapshots/counter.snap")
```

1. `literal_in_path!()` expands to code containing `include!("snapshots/counter.snap")`
2. `include!()` loads file contents: `42u32` (or empty → `Default::default()`)
3. Registry code executes with the loaded value

At compile time, the snapshot value is **baked into the binary**.

### Build Script Integration

To handle missing snapshot files on first compile:

```rust
// build.rs
fn main() {
    // Scan source for literal_in_path!() calls
    // Create missing snapshot files with default values
    // Or: create empty files that eval to Default::default()
}
```

**Alternative**: Special compilation mode that accepts defaults on first build.

### Implementation Challenges

**Compile-time vs Runtime Dilemma:**
- `include!()` needs the file to exist **at compile time**
- But snapshot files are created **at runtime** (during test execution)
- This creates a chicken-and-egg problem

**Possible Solutions:**
1. **Build script** - scan source, create missing files before compile
2. **Procedural macro** - more control, but heavier dependency
3. **Two-pass workflow** - first pass creates files, second compiles
4. **Optional files** - use `include!(concat!(...))` with fallback logic

**Complexity Assessment:**
Medium-high complexity due to compile-time/runtime boundary. Deferring to future work.

### Example Workflow (Conceptual)

```rust
#[test]
fn test_user_creation() {
    // Source stays this way - never rewritten
    // Value loaded from snapshots/user.snap (or Default::default() if empty)
    let expected = literal_in_path!("snapshots/user.snap");

    let actual = create_user("Alice");
    assert_eq!(actual, *expected);
}
```

**First compile:**
- Build script creates empty `snapshots/user.snap` if missing
- Empty file evaluates to `Default::default()`
- Test runs, `.set()` updates the snapshot file

**Subsequent compiles:**
- `include!("snapshots/user.snap")` loads the snapshot value
- Baked into binary at compile time

**On test failure:**
```bash
LITERAL_MODE=write cargo test  # Updates snapshots/user.snap
git diff snapshots/user.snap   # Review changes in external file
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
