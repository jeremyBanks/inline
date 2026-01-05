# jeb-literal Design Document

This document describes the design and architecture of jeb-literal, a self-modifying literal value system for Rust.

**Document Status**: All sections marked UNAPPROVED are pending review and approval.

---

<!-- SECTION: Overview | STATUS: UNAPPROVED -->

## Overview

jeb-literal is an experimental Rust library that provides **mutable literals** - values that can modify their own source code representation at runtime. It enables patterns like snapshot testing and self-updating configuration through a smart pointer API that treats source code as mutable state.

### Design Goals

1. **Ergonomic API**: Feel like native Rust with minimal syntax overhead
2. **Type Safety**: Leverage Rust's type system to prevent misuse
3. **Stability**: Values persist across source code changes (line insertions, etc.)
4. **Format Preservation**: Maintain original code formatting and style
5. **Thread Safety**: Support concurrent access without corruption
6. **Mode Flexibility**: Support multiple operational modes (write, verify, memory-only)

### Core Innovation

The key insight is treating source code as a **synchronized mutable store** rather than immutable text. By maintaining a registry of literal values and using character-range splicing to update source files, we create a bidirectional link between runtime values and their source representation.

<!-- END SECTION: Overview -->

---

<!-- SECTION: Core Concepts | STATUS: UNAPPROVED -->

## Core Concepts

### 1. Literal Values

A **literal value** is a Rust value that:
- Has a source code representation (via the `literal!()` macro)
- Can be read and modified at runtime
- Automatically syncs changes back to source code (in Write mode)
- Persists across function calls within the same execution

```rust
let mut counter = literal!(0u32);
*counter += 1;  // Writes "1" back to source code on drop
```

### 2. Value Identity and Stability

Each literal has a **stable identity** based on:
- **File path**: Which source file contains the literal
- **Index**: Position among all literals in that file (0th, 1st, 2nd, etc.)
- **Type**: Rust's `TypeId` for type safety

This is **more stable** than line/column positions because inserting lines above a literal doesn't change its index.

### 3. Serialization via Bake

Values must implement the `Bake` trait from the [databake](https://docs.rs/databake) crate:

```rust
pub trait Value: Bake + Clone + PartialEq {}
```

**Bake** serializes Rust values to Rust source code (token streams) for writing to source files.

**Clone** enables creating a working copy in the public `literal` field and storing an `original` value for change detection.

**PartialEq** enables detecting mutations by comparing `original == literal` on drop.

### 4. Operational Modes

The library has **four modes** controlled by the `LITERAL_MODE` environment variable:

| Mode | Behavior | Default When |
|------|----------|--------------|
| **Write** | Modifies source files | Outside tests, when run by `cargo` |
| **Verify** | Checks values match source, panics on mismatch | In tests (`#[cfg(test)]`) |
| **Memory** | Changes in-memory only, never writes | Outside tests, when not run by `cargo` |
| **Reject** | Always panics on write attempts | Never (explicit opt-in only) |

**Note:** If the default Cargo feature `"write"` is disabled, Write mode behaves the same as Memory mode.

### 5. Write-on-Drop Pattern

Mutations are detected and written **on drop**:

```rust
{
    let mut counter = literal!(0u32);
    counter.literal = 42;  // Just assigns field
    // Drop happens here → detects change → writes to file
}
```

This is enabled by:
- A public `literal: T` field for ergonomic assignment
- An `original: T` field storing the initial value
- A `Drop` impl that compares with `PartialEq` and writes if changed

<!-- END SECTION: Core Concepts -->

---

<!-- SECTION: Architecture Layers | STATUS: UNAPPROVED -->

## Architecture Layers

The system is organized into four main layers:

```
┌─────────────────────────────────────────────────┐
│  User API (literal! macro, Literal<T>)         │
├─────────────────────────────────────────────────┤
│  Registry (type-erased value storage)           │
├─────────────────────────────────────────────────┤
│  Runtime (file parsing, AST manipulation)       │
├─────────────────────────────────────────────────┤
│  File System (source code files)                │
└─────────────────────────────────────────────────┘
```

### Layer 1: User API (`src/inline.rs`)

**Components:**
- `literal!()` macro - Entry point, captures source location
- `Literal<T>` - Smart pointer wrapper, holds lock and provides API
- `LiteralInner<T>` - Internal value holder (hidden from users)

**Key Types:**

```rust
pub struct Literal<T: Value + 'static> {
    pub literal: T,                                    // Working copy
    guard: MutexGuard<'static, LiteralInner<T>>,      // Exclusive lock
    original: T,                                       // For change detection
}
```

**API Surface:**
- `literal!(expr)` - Create/access a literal value
- `literal!()` - Create with `Default::default()`
- `example.literal = value` or `*example = value` - Write via field or `DerefMut`
- `example.literal` or `*example` - Read via field or `Deref`

### Layer 2: Registry (`src/registry.rs`)

**Purpose:** Type-erased global storage for literal values with stable identity.

**Key Type:**
```rust
static VALUE_REGISTRY: Lazy<Mutex<HashMap<RegistryKey, RegistryValue>>>

type RegistryKey = (PathBuf, u32, u32, TypeId);  // (file, line, column, type)
type RegistryValue = usize;  // Raw pointer to Box<Mutex<LiteralInner<T>>>
```

**Responsibilities:**
- De-duplicate literals at the same source location
- Provide `'static` lifetime through intentional memory leaks
- Ensure type safety via `TypeId` in keys
- Use compile-time (line, column) as identity (never changes)

**Note:** Stable index resolution happens lazily on first write/verify, not during registry lookup.

**Safety invariants:**
- Pointers are never freed (intentional leak for `'static` lifetime)
- `TypeId` guarantees we only cast to the correct type
- Boxes are only created by this module

### Layer 3: Runtime (`src/runtime.rs`)

**Purpose:** File I/O, parsing, AST manipulation, and caching.

**Key Types:**
```rust
struct SharedState {
    source: String,        // Current source (may differ from disk)
    version: u64,          // Incremented on each modification
    disk_source: String,   // Last known disk state
}

struct FileState {
    shared: Arc<RwLock<SharedState>>,  // Shared across threads
    path: PathBuf,
}

thread_local! {
    static CACHE: RefCell<HashMap<PathBuf, CachedState>>
}
```

**Responsibilities:**
- Parse source files to AST (`syn::File`)
- Build position-to-index mapping
- Detect concurrent external modifications via `disk_source` comparison
- Verify initial value matches source on first access in Verify mode
- Perform character-range splicing for updates
- Cache parsed ASTs per-thread
- Write modified source to disk

**Locking strategy:**
- `RwLock` for `SharedState` - allows concurrent readers
- `Mutex` for `VALUE_REGISTRY` - simpler, write-heavy
- Thread-local caching required because AST types aren't `Send`

### Layer 4: File System

**Source files** are only accessed when writing or verifying changes. The library:
- Parses source files lazily when needed to write or verify (never during normal reads)
- Verifies initial value matches source in Verify mode (when creating a new registry entry and mode requires verification)
- Detects concurrent external modifications before writing (via `disk_source` comparison)
- Preserves formatting via character-range splicing when writing
- Only attempts writes when running under `cargo`

<!-- END SECTION: Architecture Layers -->

---

<!-- SECTION: Data Flow | STATUS: UNAPPROVED -->

## Data Flow

### Read Path (Accessing a Literal)

```
User code: literal!(42u32)
    ↓
Macro expansion: registry::get_or_create(42, file!(), line!(), column!())
    ↓
Registry: Check for existing (file, line, column, TypeId) → found?
    ├─ YES → Return &'static Mutex<LiteralInner<T>>
    └─ NO  → Create Box<Mutex<LiteralInner<T>>>, leak for 'static
             Insert into global map with key (file, line, column, TypeId)
             Return &'static Mutex<LiteralInner<T>>
    ↓
Macro: Call .lock() → MutexGuard<LiteralInner<T>>
    ↓
Literal::from_guard(): Clone value twice (working + original)
    ↓
Return Literal<T> to user
```

**Note:** No file I/O occurs during reads. The registry uses compile-time (line, column) as identity.

### Write Path (Modifying a Literal)

**Via write-on-drop:**

```
User code: counter.literal = 42
    ↓
Field assignment: self.literal = 42 (in-memory only)
    ↓
[...later...]
    ↓
Drop::drop(&mut self)
    ↓
Compare: original == literal?
    ├─ YES → No-op, return early
    └─ NO  → Continue to write logic
        ↓
    Check mode:
        ├─ Memory  → Update guard.value, return (no file I/O)
        ├─ Reject  → Return silently
        ├─ Verify  → Resolve index → verify_source() → panic if mismatch
        └─ Write   → Resolve index → update_source() → write_to_disk()
            ↓
        Runtime: Bake value to tokens
        Runtime: Update SharedState.source (character-range splice)
        Runtime: Increment SharedState.version
        Runtime: Write to disk
        Runtime: Update SharedState.disk_source
```

### Index Resolution (Lazy, Cached)

```
get_macro_index(file, line, column)
    ↓
FileState: Get or create for file
    ↓
Check thread-local cache: version matches?
    ├─ YES → Use cached (ast, position_to_index)
    └─ NO  → Parse SharedState.source → syn::File
             Walk AST, count literal!() macros, build position map
             Cache (ast, position_to_index, version)
    ↓
Look up (line, column) in position_to_index
    ↓
Return stable index
```

<!-- END SECTION: Data Flow -->

---

<!-- SECTION: Key Design Decisions | STATUS: UNAPPROVED -->

## Key Design Decisions

### 1. Registry Key Uses (line, column), Index Resolved Lazily

**Decision:** Use compile-time (file, line, column) as registry key identity. Resolve stable index only when writing/verifying.

**Rationale:**
- Compile-time (line, column) from `file!()`, `line!()`, `column!()` never changes
- Eliminates file parsing on reads - just hash map lookup
- Index only needed when actually modifying source files
- Index resolution can be lazy and cached per-file

**Trade-off:**
- Index still needed for writing (to find Nth literal in file after edits)
- But reads are faster with no file I/O
- Adding/removing literals still shifts later indices (acceptable for snapshot testing)

### 2. Intentional Memory Leaks for `'static`

**Decision:** Leak `Box<Mutex<LiteralInner<T>>>` to get `'static` lifetime.

**Rationale:**
- Registry must return `&'static` references
- Values must persist for entire program duration
- No safe way to prove exclusive ownership for deallocation

**Trade-off:**
- Memory grows with number of unique literal locations
- Acceptable for typical usage (10s-100s of literals, not millions)

### 3. Write-on-Drop Instead of Explicit `.set()`

**Decision:** Detect mutations in `Drop` by comparing with `PartialEq`.

**Rationale:**
- More ergonomic: `counter.literal = 42` vs `counter.set(42)`
- Supports both direct field assignment and `DerefMut` (`*counter = 42`)
- Consistent with Rust's RAII philosophy

**Trade-off:**
- Requires `Clone` bound (to store `original` value)
- Requires `PartialEq` bound (to detect changes)
- One extra clone per literal creation
- Writes happen at drop time (requires explicit scopes in some tests)

### 4. Public `literal` Field (Not `value`)

**Decision:** Name the public field `literal` instead of `value`.

**Rationale:**
- Reduces collision chance with inner type methods
- If `T` has a `.value` field/method, `Deref` would conflict
- Clearer intent: this is the literal's value

**Trade-off:**
- Slightly more verbose: `counter.literal` vs `counter.value`
- But avoids subtle bugs from name collisions

### 5. PartialEq-Based Comparison for Change Detection

**Decision:** Compare values with `PartialEq` to detect changes, use `Bake` only for serialization.

**Rationale:**
- More intuitive: standard Rust equality semantics
- Most types already implement `PartialEq`
- Clear separation: `PartialEq` for change detection, `Bake` for serialization

**Trade-off:**
- Requires both `PartialEq` and `Bake` bounds
- Types without `PartialEq` cannot be used (rare)

### 6. Mode-Based Behavior with Cargo Detection and Feature Flag

**Decision:** Use runtime `LITERAL_MODE` env var, default based on cargo detection, with optional compile-time feature flag.

**Rationale:**
- Same binary can verify or update snapshots via env var
- Easier for users: `LITERAL_MODE=write cargo test`
- Safe defaults: Memory mode when not run by `cargo`, Write mode when run by `cargo`
- Optional "write" feature can be disabled to compile out write functionality

**Trade-off:**
- Runtime overhead (mode check on every mutation)
- Can't dead-code eliminate write logic unless feature disabled
- Acceptable: mode check is trivial compared to file I/O

### 7. Character-Range Splicing (Not Pretty-Printing)

**Decision:** Replace only the macro's token span, preserving surrounding code.

**Rationale:**
- Maintains original formatting, comments, whitespace
- Minimal diff in version control
- Predictable output

**Trade-off:**
- More complex implementation (span tracking)
- Long values may wrap lines (handled by prettyplease for new content)

### 8. Mutex for Values, RwLock for Files

**Decision:** Different lock types for different granularities.

**Rationale:**
- `Mutex<LiteralInner<T>>`: Exclusive access model (always modify)
- `RwLock<SharedState>`: Many readers, occasional writers (file state)
- Optimizes for common case (many literals, few file updates)

**Trade-off:**
- More complexity (two lock types)
- Better performance under concurrent access

### 9. Thread-Local AST Caching

**Decision:** Each thread caches its own parsed AST.

**Rationale:**
- AST types (`syn::File`) aren't `Send`, cannot be shared across threads
- Version-based invalidation ensures consistency across threads

**Trade-off:**
- Memory usage scales with thread count
- Acceptable for typical usage (few threads, small ASTs)

<!-- END SECTION: Key Design Decisions -->

---

<!-- SECTION: Use Cases | STATUS: UNAPPROVED -->

## Use Cases

### 1. Snapshot Testing

**Problem:** Manually updating test expectations is tedious.

**Solution:**
```rust
#[test]
fn test_render_output() {
    let output = render_component();
    let expected = literal!("expected HTML".to_string());

    assert_eq!(output, *expected);
    // First run: write mode updates expected to match output
    // Subsequent runs: verify mode checks output == expected
}
```

**Workflow:**
```bash
LITERAL_MODE=write cargo test  # Update all snapshots
cargo test                      # Verify snapshots match
```

### 2. Self-Updating Run Counter

**Problem:** Track how many times a program has run without external files.

**Solution:**
```rust
fn main() {
    let mut counter = literal!(0u32);
    println!("This program has run {} times", *counter + 1);
    *counter += 1;  // Source code now contains new count
}
```

### 3. Development-Time Configuration

**Problem:** Tweaking configuration values during development.

**Solution:**
```rust
fn dev_server() {
    let mut config = literal!(ServerConfig {
        port: 8080,
        debug: true,
    });

    // Modify config at runtime, persists to source
    config.literal.port = 3000;
    // Next run starts with port 3000
}
```

### 4. Interactive Data Exploration

**Problem:** Refining data transformations through iteration.

**Solution:**
```rust
let mut filters = literal!(vec!["spam", "ads"]);
let cleaned = data.filter(|item| {
    !filters.iter().any(|f| item.contains(f))
});

// Discover new filter needed
filters.literal.push("promotions");
// Re-run with updated filters without editing code
```

### 5. Regression Test Generation

**Problem:** Capturing actual output as expected values.

**Solution:**
```rust
#[test]
fn test_parse_response() {
    let response = fetch_api_response();
    let parsed = parse(response);
    let snapshot = literal!(ParsedResponse::default());

    // First run in write mode: captures actual parsed value
    // Future runs: verify parsing stays consistent
    assert_eq!(parsed, *snapshot);
}
```

<!-- END SECTION: Use Cases -->

---

<!-- SECTION: Performance Considerations | STATUS: UNAPPROVED -->

## Performance Considerations

### Hot Paths

**Accessing a literal (already in registry):**
- Registry lookup: `O(1)` hash map access
- Mutex lock: `O(1)` uncontended (typically)
- Clone value twice: `O(n)` in value size
- **Total: Fast** for small values, scales with value size

**Detecting no-op writes:**
- Compare with `PartialEq`: `O(n)` in value size (depends on type)
- **Total: Fast for most types**, avoids unnecessary file I/O

### Cold Paths

**First access to a literal (not in registry):**
- Create registry entry: `O(1)`
- No file I/O on reads
- **Total: Fast**, just hash map insertion

**Writing to source:**
- Resolve index (may parse): See above
- Character-range splice: `O(n)` in file size
- Write to disk: I/O-bound
- **Total: Expensive, but only on mutation**

### Caching Strategy

**Thread-local AST cache:**
- Avoids re-parsing on every index lookup
- Invalidated by version counter
- Trade-off: Memory for speed

**Shared source state:**
- `Arc<RwLock<SharedState>>` shared across threads
- Read-heavy workload (many literals, few writes)
- RwLock allows concurrent readers

### Scalability Limits

**Number of literals:** Grows registry size (leaked memory)
- 10,000 literals × 1KB each = 10MB (fine)
- 1,000,000 literals = 1GB (problematic)

**File size:** Affects parse time and splice time
- 10,000 line file: ~100ms parse time (acceptable)
- 100,000 line file: ~1s parse time (slow)

**Concurrent access:** Mutex contention on high write load
- Read-heavy: Scales well (RwLock for files)
- Write-heavy: Serialized by Mutex (acceptable for typical use)

### Optimization Opportunities

**Future improvements:**
1. Incremental parsing (reparse only changed regions)
2. Persistent AST cache (across runs)
3. Lazy value cloning (clone-on-write)
4. Batch writes (flush multiple changes at once)

<!-- END SECTION: Performance Considerations -->

---

<!-- SECTION: Safety and Limitations | STATUS: UNAPPROVED -->

## Safety and Limitations

### Memory Safety

**Unsafe code:**
- `src/registry.rs`: Pointer casting for type erasure
- Invariants: TypeId ensures correct type, pointers never freed

**Safety guarantees:**
- ✅ No data races (Mutex/RwLock synchronization)
- ✅ No use-after-free (intentional leaks)
- ✅ No type confusion (TypeId in registry keys)

### Soundness Issues

**Potential unsoundness:**
- ❌ None known in current design

**Assumptions:**
- Single process modifies each file (no external editors)
- Source files remain valid Rust
- Literal count remains stable (adding/removing shifts indices)

### Limitations

**1. File System**
- Requires running under `cargo` (checks for cargo env vars)
- No write access outside cargo environments
- File must be parseable Rust code

**2. Type System**
- `T: Bake + Clone + PartialEq + 'static` bound
- Not all types implement `Bake` (most primitives and std types do)
- Types without `PartialEq` cannot be used
- Custom types need manual `Bake` impl or derive

**3. Concurrency**
- Same literal from multiple threads: Last writer wins
- No transaction semantics
- No conflict resolution (accept it or use Mutex externally)

**4. Index Stability**
- Adding literal above shifts indices of literals below
- Removing literal shifts all subsequent indices
- Acceptable for snapshot testing (update in write mode)

**5. Value Size**
- Large values cause slow baking/cloning
- Very long values may wrap lines in source
- No hard limit, but impractical above ~10KB

**6. Error Handling**
- Drop cannot return errors (panics or silent failure)
- Parse errors surface as panics in write mode
- File I/O errors silently ignored in drop (cannot panic)

### Anti-Patterns

**Don't:**
- ❌ Use for production data storage (experimental, modifies source)
- ❌ Store secrets in literals (they'll be in source code!)
- ❌ Use for very large values (performance degrades)
- ❌ Rely on literal order (indices shift when literals added)
- ❌ Edit source files externally while running (changes may be lost)

**Do:**
- ✅ Use for snapshot testing in tests
- ✅ Use for development-time configuration
- ✅ Run in verify mode in CI
- ✅ Commit source files with literals to version control
- ✅ Use write mode locally to update snapshots

### Platform Support

**Supported:**
- ✅ Linux, macOS, Windows (anywhere `parking_lot` works)
- ✅ All Rust editions (tested on 2021)

**Unsupported:**
- ❌ No-std (requires file I/O, parking_lot)
- ❌ WASM (no file system access)

<!-- END SECTION: Safety and Limitations -->

---

<!-- SECTION: Future Directions | STATUS: UNAPPROVED -->

## Future Directions

### Planned Features

**1. Extension Trait for `.set()`**
- Optional `LiteralExt` trait with explicit `.set()` method
- Avoid auto-deref pollution while offering alternative syntax
- Use case: Prefer `counter.set(42)` over `counter.literal = 42`

**2. Serde Compatibility**
- Support any `Serialize + Deserialize` type (beyond `Bake`)
- Serialize to `ron` or `json`, embed in source as string literal
- Broader type support at cost of less idiomatic Rust

**3. File-Backed Literals**
- Store snapshots in separate `.snap` files (like `insta` crate)
- Stable identifiers independent of line numbers
- Better for very large snapshots

**4. Tooling Integration**
- `cargo-literal` command for reviewing snapshot changes
- Interactive mode like `git add -p`
- Diff viewer for changed snapshots

**5. Incremental Parsing**
- Track changed regions, reparse only affected spans
- Avoid full file parse on every write
- Significant perf win for large files

### Research Directions

**1. Persistent AST Cache**
- Serialize parsed AST to disk
- Invalidate based on file mtime
- Avoid parse overhead across runs

**2. Batch Write Mode**
- Collect multiple changes, write once at end
- Reduce file I/O overhead
- Requires explicit flush or atexit hook

**3. Transaction Semantics**
- Group multiple literal updates
- All-or-nothing write (rollback on error)
- Useful for multi-file snapshots

**4. Conflict Resolution**
- Detect concurrent modifications
- User-defined merge strategies
- Integrate with version control

**5. IDE Integration**
- Language server protocol extension
- Highlight literals that have changed
- Quick-fix to accept/reject changes

### Open Questions

**1. Macro syntax:**
- Support `literal! { value }` in addition to `literal!(value)`?
- Named literals: `literal!(counter: 0)`?

**2. Scoping:**
- Should literals be scoped to module, file, or global?
- Current: global per-file, indexed by position

**3. Versioning:**
- How to handle snapshot format changes across versions?
- Migration path for incompatible changes?

**4. Testing:**
- How to test the library itself without self-modification?
- Current: Fixture files, careful test isolation

<!-- END SECTION: Future Directions -->

---

<!-- SECTION: Comparison to Related Work | STATUS: UNAPPROVED -->

## Comparison to Related Work

### vs. `expect-test`

**Similarities:**
- Both enable snapshot testing in Rust
- Both update source files with expected values
- Both use character-range splicing

**Differences:**
- `expect-test`: String-based, uses `expect![[...]]` syntax
- `jeb-literal`: Typed values, uses `literal!(value)` syntax
- `jeb-literal`: Supports non-test use cases (self-modifying code)
- `jeb-literal`: Persistence across runs (registry)

**Inspiration:**
jeb-literal is heavily based on `expect-test`, extending the concept to typed values and runtime persistence.

### vs. `insta`

**Similarities:**
- Snapshot testing for Rust
- Supports multiple modes (update/verify)
- Type-aware snapshots

**Differences:**
- `insta`: External `.snap` files
- `jeb-literal`: Inline in source code
- `insta`: Review workflow with `cargo insta review`
- `jeb-literal`: Simpler (just environment variable)

**Trade-offs:**
- `insta`: Better for large snapshots, stable file layout
- `jeb-literal`: Better for small values, no extra files

### vs. Self-Modifying Code (General)

**Traditional approaches:**
- Write to separate data files
- Code generation tools (build scripts)
- Configuration management systems

**jeb-literal advantages:**
- No external files needed
- Type-safe, compiler-checked values
- Integrates with Rust's ownership system
- Version control friendly (values in source)

**jeb-literal disadvantages:**
- Modifying source is unconventional
- Requires careful mode management
- Not suitable for production deployment

<!-- END SECTION: Comparison to Related Work -->

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-01-04 | Initial draft, all sections unapproved |

