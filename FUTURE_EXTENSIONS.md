# Future Extensions & Ideas

This document captures rough ideas for potential future extensions to inline. These are **not currently planned for implementation** but are noted for future exploration.

## 1. Alternative Serialization Formats (e.g., JSON, TOML, YAML)

**User's idea**: Support serde_json for storing values as JSON strings instead of Rust code.

**Example**:
```rust
let config: Config = json_litter!(r#"{"host": "localhost", "port": 8080}"#);
config.set(Config { host: "prod.example.com".into(), port: 443 });
// File updates to: json_litter!(r#"{"host": "prod.example.com", "port": 443}"#)
```

**Tentative implementation notes** (Claude):
- New trait: `trait JsonLiteral: Serialize + Deserialize {}`
- New struct: `JsonLitter<T>` (parallel to `Inline<T>`)
- Different macro: `json_litter!()` stores string literal containing JSON
- Same architecture: index-based AST anchoring, same mode system
- Compare serialized JSON strings instead of Rust AST tokens
- Runtime deserializes on creation from string literal

**Challenges**:
- JSON lacks Rust type information (need type annotations/generics)
- Error handling for malformed JSON at runtime
- Formatting choices: pretty-print vs compact
- Similar extensions possible for TOML, YAML, etc.

**Status**: Feasible extension using same core architecture

---

## 2. Type Annotations / Explicit Type Parameters

**User's idea**: Support statically defining types for values, especially useful for formats like JSON where type can't be inferred from data.

**Approaches discussed**:

### A. Type inference (already works)
```rust
let config: Inline<Config> = inline!(Config::default());
```

### B. Turbofish syntax (requires proc macro)
```rust
let config = inline::<Config>(Config::default());
```

### C. Type as first parameter (for typed variants)
```rust
let config = json_litter!(Config, r#"{"x": 1}"#);
// Expands to: JsonLitter::<Config>::__new_from_json(...)
```

**Tentative notes** (Claude):
- Option C makes most sense for JSON/typed variants
- Turbofish requires proc macro instead of macro_rules
- Type parameter necessary for deserialization in JSON case

**Status**: Design question - depends on which formats we support

---

## 3. Empty Macro / Placeholder Pattern

**User's idea**: Support `inline!()` with no arguments as a placeholder that gets filled in on first write.

**Example**:
```rust
// Write in source initially:
let config: Inline<Config> = inline!();

// First set() rewrites source to:
let config: Inline<Config> = inline!(Config { ... });
```

**Tentative implementation notes** (Claude):
```rust
macro_rules! inline {
    () => {{
        $crate::Inline::__new(Default::default(), file!(), line!(), column!())
    }};
    ($value:expr) => {{
        $crate::Inline::__new($value, file!(), line!(), column!())
    }};
}
// Would require: pub trait Literal: Bake + Default {}
```

**Use cases**:
- Scaffolding new config values
- "TODO: configure this" markers in code
- Less boilerplate when adding new litters

**Challenges**:
- Requires `Default` trait (not all types have it)
- Less readable - `inline!()` doesn't show what type it is
- File state shows `inline!()` initially, then gets replaced

**Status**: Feasible, good for scaffolding workflow

---

## 4. Caller Location Without Macros

**User's idea**: Investigate if we can avoid macros entirely using Rust's `#[track_caller]` and `std::panic::Location` to find call sites and replace them.

**Example** (hypothetical):
```rust
#[track_caller]
fn inline<T>(value: T) -> Inline<T> {
    let location = std::panic::Location::caller();
    // Find and replace the function call in source?
    Inline::__new(value, location.file(), location.line(), location.column())
}
```

**Questions to investigate**:
- Can we reliably find the exact call site in the AST?
- Can we distinguish between different calls on the same line?
- Does `Location` give us enough precision?
- Was this proven viable or not in earlier exploration?

**Status**: TO BE INVESTIGATED - user doesn't remember if this was proven viable

---

## 5. At-Exit Hooks for Deferred Writes

**Context**: Early design had references/locks, needed at-exit hooks to flush pending changes.

**Current status**: **Out of scope by design**
- Writes are immediate (every `set()` writes to disk)
- No buffering or deferred writes
- No need for hooks since there's nothing to flush

**Why mentioned**: Historical context - was part of earlier design with different concurrency model.

**Status**: Not applicable to current immediate-write architecture

---

## 6. Multi-Threaded Support ✅ **IMPLEMENTED**

**Initial status**: `thread_local!` only - no cross-thread coordination.

**Challenge**: `proc_macro2` and `syn` types are intentionally NOT Send/Sync.

**Test results**:
```rust
// These fail to compile:
assert_send::<proc_macro2::Span>();      // ✗ not Send
assert_sync::<proc_macro2::Span>();      // ✗ not Sync
assert_send::<syn::File>();              // ✗ not Send
assert_sync::<syn::File>();              // ✗ not Sync
```

**Root cause**: `proc_macro2` uses `PhantomData<Rc<()>>` to match real `proc_macro` thread-safety.

**Solution implemented**: ✅ **Hybrid lock-based architecture**

### Architecture

```rust
// Shared state across threads (Send+Sync)
Arc<RwLock<SharedState>> {
    source: String,        // Canonical prettyplease output
    disk_source: String,   // What's actually on disk (after cargo fmt)
    version: u64,          // Incremented on every modification
}

// Thread-local cache (!Send !Sync)
thread_local! {
    CACHE: HashMap<Path, CachedState> {
        ast: syn::File,              // Parsed AST (cached)
        position_to_index: HashMap,  // Position mapping
        version: u64,                // Cache validity check
    }
}
```

### Features

✅ **Multi-threaded reads**: Fast - uses thread-local cached AST
✅ **Multi-threaded writes**: Write lock held for entire operation (prevents concurrent modifications)
✅ **Cross-thread visibility**: Changes from one thread visible to others
✅ **Multi-process detection**: Panics with clear error if external process modifies file
✅ **Position stability**: `shared.source` is canonical for line/column lookups (unaffected by cargo fmt)

### Performance Trade-offs

- **Cost**: Re-parses AST when thread switches or sees new version
- **Benefit**: Correctness across threads without Send/Sync violations
- **Acceptable**: For scripts and tests, this overhead is minimal

### Testing Notes

- Tests pass with `--test-threads=1`
- Parallel test failures due to shared environment variables (`INLINE_MODE`)
- **Recommended**: Use `cargo nextest` for parallel testing (runs each test in separate process)

**Status**: ✅ **IMPLEMENTED** - Fully supports multi-threaded and multi-process usage

---

## Notes

- These are rough ideas captured for later evaluation
- Implementation details are tentative first reactions from Claude
- Actual design would require deeper exploration
- Priorities TBD based on actual use cases
