# Future Extensions & Ideas

This document captures rough ideas for potential future extensions to litter. These are **not currently planned for implementation** but are noted for future exploration.

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
- New struct: `JsonLitter<T>` (parallel to `Litter<T>`)
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
let config: Litter<Config> = litter!(Config::default());
```

### B. Turbofish syntax (requires proc macro)
```rust
let config = litter::<Config>(Config::default());
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

**User's idea**: Support `litter!()` with no arguments as a placeholder that gets filled in on first write.

**Example**:
```rust
// Write in source initially:
let config: Litter<Config> = litter!();

// First set() rewrites source to:
let config: Litter<Config> = litter!(Config { ... });
```

**Tentative implementation notes** (Claude):
```rust
macro_rules! litter {
    () => {{
        $crate::Litter::__new(Default::default(), file!(), line!(), column!())
    }};
    ($value:expr) => {{
        $crate::Litter::__new($value, file!(), line!(), column!())
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
- Less readable - `litter!()` doesn't show what type it is
- File state shows `litter!()` initially, then gets replaced

**Status**: Feasible, good for scaffolding workflow

---

## 4. Caller Location Without Macros

**User's idea**: Investigate if we can avoid macros entirely using Rust's `#[track_caller]` and `std::panic::Location` to find call sites and replace them.

**Example** (hypothetical):
```rust
#[track_caller]
fn litter<T>(value: T) -> Litter<T> {
    let location = std::panic::Location::caller();
    // Find and replace the function call in source?
    Litter::__new(value, location.file(), location.line(), location.column())
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

## 6. Thread-Local Storage ~~Removal~~ (Investigated - Not Possible)

**Current status**: Using `thread_local!` for file state storage.

**Question investigated**: Can we remove `thread_local!` and use `static Arc<RwLock<...>>` instead?

**Finding**: ❌ **Not possible with current dependencies**

**Test results**:
```rust
// These fail to compile:
assert_send::<proc_macro2::Span>();      // ✗ not Send
assert_sync::<proc_macro2::Span>();      // ✗ not Sync
assert_send::<syn::File>();              // ✗ not Send
assert_sync::<syn::File>();              // ✗ not Sync
assert_send::<proc_macro2::TokenStream>(); // ✗ not Send
assert_sync::<proc_macro2::TokenStream>(); // ✗ not Sync
```

**Root cause**: `proc_macro2` intentionally uses `PhantomData<Rc<()>>` to make its types NOT Send/Sync, even outside proc macro context. This is deliberate design to match the thread-safety characteristics of real `proc_macro` types and catch bugs early.

**Implications**:
- ✅ `thread_local!` is the correct choice
- ⚠️ Each thread has its own FileState cache
- ⚠️ No cross-thread coordination possible
- ⚠️ Two threads writing same file = race condition
- ✅ For single-threaded scripts (main use case), this is fine

**Alternative explored**: Could we use a different AST library?
- Would need to parse/format Rust code without using syn/proc-macro2
- No viable alternatives with similar functionality
- Not worth the effort for this use case

**Status**: RESOLVED - `thread_local!` is necessary and correct

---

## Notes

- These are rough ideas captured for later evaluation
- Implementation details are tentative first reactions from Claude
- Actual design would require deeper exploration
- Priorities TBD based on actual use cases
