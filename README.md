# jeb-literal

`jeb-literal` provides mutable literals as smart pointers into your source code.

## Overview

jeb-literal lets you create values that can update themselves in your source code. This is an experimental approach to snapshot testing and self-modifying code in Rust.

```rust
use jeb_literal::literal;

fn main() {
    let mut counter = literal!(0u32);

    println!("Run #{}", *counter + 1);

    *counter += 1;  // Mutate directly - writes on drop!
    // The source file is now updated with the new value!
}
```

## Features

- **Self-Modifying Code**: Values that update their own source code
- **Write-on-Drop**: Mutations through `DerefMut` automatically persist
- **Default Values**: `literal!()` with no arguments uses `Default::default()`
- **Type-Safe**: Uses Rust's type system and databake for serialization
- **Mode-Based**: Control when updates happen via environment variables
- **Stable Positions**: Index-based tracking survives line insertions
- **Thread-Safe**: File-level locking prevents corruption
- **Format-Preserving**: Character-range splicing preserves original formatting

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
jeb-literal = "0.0.1-dev.1"
```

Create a literal value:

```rust
use jeb_literal::literal;

let mut value = literal!(42u32);
let counter: Literal<u32> = literal!();  // Uses Default::default()
```

### Reading Values

There are multiple equivalent ways to read a literal value:

```rust
let counter = literal!(42u32);

// Preferred: use the dereference operator
let x = *counter;           // Most idiomatic
let x = &*counter;          // When you need a reference

// Alternative: direct field access
let x = counter.literal;

// Alternative: explicit method (less common)
let x = *counter.get();
```

### Writing Values

Update it using either field assignment or `DerefMut`:

```rust
// Option 1: Direct field assignment (explicit)
value.literal = 100u32;

// Option 2: DerefMut syntax (works for compound operations)
*value = 100u32;
*value += 1;

// Both syntaxes write to your source file on drop (in Write mode)!
```

**When to use which:**
- Use `*counter` for simple reads and writes
- Use `counter.literal` when you want to emphasize mutation
- For nested field access: `counter.literal.nested_field = x`

### Old Style (Deprecated Pattern)

Some examples may use `.get()` which is redundant with `Deref`:

```rust
// Old style (verbose)
let x = *counter.get();

// New style (preferred)
let x = *counter;
```

## Modes

jeb-literal has four modes, controlled by the `LITERAL_MODE` environment variable:

| Mode | Behavior | Default When |
|------|----------|--------------|
| **Write** | Changes are written back to source files | Outside tests, when running under `cargo` |
| **Verify** | Validates that values round-trip correctly, panics on mismatch | In tests (`#[cfg(test)]`) |
| **Memory** | Changes in memory only, no file writes | Outside tests, NOT running under `cargo` |
| **Reject** | Rejects any write attempts, always fails | Never (explicit opt-in via env var) |

### Default Mode Logic

The default mode is **context-dependent** for safety:

1. **In `#[test]` functions**: Defaults to **Verify** mode (for snapshot testing)
2. **Outside tests, under cargo**: Defaults to **Write** mode (checks for `CARGO` env vars)
3. **Outside tests, NOT under cargo**: Defaults to **Memory** mode (safeguard against accidental writes)

This means compiled binaries default to Memory mode unless explicitly set to Write.

### Usage Examples

```bash
LITERAL_MODE=write cargo run          # Enable self-modifying mode
LITERAL_MODE=memory cargo run         # Run without file writes
cargo test                            # Verify mode (default in tests)
LITERAL_MODE=write cargo test         # Update all snapshots
LITERAL_MODE=reject cargo test        # Ensure no writes attempted
```

**Note:** If the `"write"` Cargo feature is disabled, Write mode behaves the same as Memory mode (writes are compiled out).

### Cargo Subcommand

For convenience, install the `cargo regenerate-test-literals` subcommand:

```bash
cargo install --path . --bin cargo-regenerate-test-literals
```

This installs a cargo extension located in `src/bin/cargo-regenerate-test-literals.rs` that wraps `LITERAL_MODE=write cargo test` for convenience.

Once installed, regenerate all test snapshots easily:

```bash
cargo regenerate-test-literals                  # Update all snapshots
cargo regenerate-test-literals -- --test-threads=1  # Run serially
cargo regenerate-test-literals test_name        # Update specific test
```

This is equivalent to `LITERAL_MODE=write cargo test` but easier to remember and type.

## How It Works

1. The `literal!()` macro captures the source location (file, line, column)
2. Values implement the `Bake` trait from [databake](https://docs.rs/databake) for serialization
3. The registry uses **index-based keys** (Nth literal in file) for stability across line insertions
4. When mutated (via `.literal =` field or `DerefMut`), jeb-literal:
   - Detects the change (using `PartialEq`)
   - Parses the source file
   - Finds the macro by its stable index
   - Uses character-range splicing to replace only the macro's value
   - Writes the file back (original formatting is preserved)

## Supported Types

Any type implementing `Bake + Clone + PartialEq` works with jeb-literal:

### What is Bake?

[`Bake`](https://docs.rs/databake) is a trait from the `databake` crate that serializes Rust values to Rust source code (token streams), not runtime formats like JSON. It's specifically designed for code generation.

### Built-in Support

Most primitive types and standard library types already implement `Bake`:

- **Primitives**: `u32`, `i64`, `f32`, `f64`, `bool`, `char`, etc.
- **Strings**: `String`, `&'static str`
- **Collections**: `Vec<T>`, arrays `[T; N]`, tuples, `Option<T>`, `Result<T, E>`
- **Other std types**: See [databake's documentation](https://docs.rs/databake) for the full list

**Requirements explained:**
- **`Clone`**: Required for write-on-drop functionality (we store both working and original copies)
- **`PartialEq`**: Used for change detection (comparing `original == literal` on drop)
- **`Bake`**: Used for serialization to Rust source code

### Custom Types

For custom types, use databake's derive macro:

```rust
use databake::*;

#[derive(Clone, PartialEq, Bake)]
#[databake(path = my_crate)]  // Specify the import path for generated code
pub struct Config {
    pub port: u16,
    pub host: String,
}

// Now Config can be used with literal!()
let config = literal!(Config { port: 8080, host: "localhost".to_string() });
```

**Note**: `Bake` generates Rust code, not runtime serialization. The value `42u32` becomes the token stream `42u32`, and `vec![1,2,3]` becomes `vec![1i32, 2i32, 3i32]` (or `alloc::vec![...]` depending on context).

### Future Ideas

**Serde Compatibility**: Add support for any type implementing `Serialize + Deserialize`, expanding beyond databake's current type coverage.

**Tooling Integration**: A `cargo-literal` command for reviewing and accepting snapshot changes interactively, similar to `git add -p`.

**File-Backed Literals**: Support external snapshot files for better organization and stability. This would provide stable identifiers independent of line numbers, but requires careful design around compile-time vs runtime tradeoffs.

**Extension Trait for `.set()`**: Provide an opt-in `LiteralExt` trait with a `.set()` method for cases where direct field assignment isn't preferred. This would avoid polluting the method namespace via auto-deref while still offering explicit setter syntax when desired.

## Examples

See the `examples/` directory for complete examples:

- `counter.rs`: Self-incrementing run counter
- `config.rs`: Self-updating configuration

## Use Cases

- Snapshot testing for complex data structures
- Counters and state that persists between test runs
- Self-updating configuration values during development
- Experimental self-modifying code patterns

## When Not to Use

This library is **experimental** and not suitable for:

- ❌ **Production applications** - Modifying source code at runtime is unconventional and fragile
- ❌ **Security-sensitive contexts** - Values are written to source files in plain text
- ❌ **Large-scale code generation** - Memory usage scales with unique literal locations (intentional leaks)
- ❌ **Distributed systems** - No synchronization across processes or machines
- ❌ **CI/CD without care** - Requires write permissions to source files

**Instead, consider:**
- For snapshot testing: Use [`insta`](https://docs.rs/insta) crate (external .snap files)
- For configuration: Use proper config files (TOML, JSON, etc.)
- For state persistence: Use databases or proper state management
- For test fixtures: Use data files or embedded resources

Note: This is an experimental library. Production use is not recommended.

## Implementation Details

### Architecture

- Per-file shared state with thread-local caching for performance
- Character-range splicing preserves original formatting
- Uses `syn` for parsing, `proc-macro2` for span locations
- Thread-safe with RwLock synchronization
- Detects concurrent external file modifications

### Assumptions

- Single process modifies each file (no external editors while updating)
- Source files are valid Rust that can be parsed
- Literal count remains stable (adding/removing literals changes indices)
- Only value content changes (the literal's position in file remains the same)

### Memory Considerations

Each unique `literal!()` source location permanently allocates memory:
- ~32 bytes overhead per literal (Box + Mutex + metadata)
- Plus `sizeof::<T>()` for the stored value

**Memory usage examples:**
- 100 literals with u32 values: ~4 KB (negligible)
- 1,000 literals with small structs (~100 bytes each): ~132 KB (acceptable)
- 100,000 literals (generated code): ~3-10 MB (may be concerning)

**Why permanent allocation?**
The registry intentionally leaks memory to provide `'static` lifetime guarantees.
This is by design and necessary for the architecture.

**For large-scale generated code**, consider:
- Using external snapshot files instead (see `insta` crate)
- Limiting literals to test code only
- Using runtime configuration for generated code

## Testing

Tests are organized into two categories based on their concurrency safety:

### Parallel-Safe Tests

These tests don't use environment variables and can run concurrently:
- `concurrent_process_detection.rs` - Tests file locking across processes
- `multi_threaded.rs` - Tests thread-safe access to literals

Run with:

```bash
cargo test --test concurrent_process_detection --test multi_threaded
```

### Serial Tests

Tests with `_serial` in their filename use the `LITERAL_MODE` environment variable and must run serially to avoid race conditions. These include:
- `default_values_serial_test.rs`
- `deref_mut_serial_test.rs`
- `index_stability_serial_test.rs`
- `integration_serial_test.rs`
- And all other `*_serial_test.rs` files

Run all tests serially (recommended):

```bash
cargo test -- --test-threads=1
```

Run a specific serial test:

```bash
cargo test --test integration_serial_test -- --test-threads=1
```

### Why Serial?

Environment variables like `LITERAL_MODE` are process-global, not thread-local. If multiple tests running in parallel both set `LITERAL_MODE`, they would interfere with each other, causing flaky failures. Running with `--test-threads=1` ensures tests execute one at a time.

### Test Fixtures

The `tests/fixtures/` directory contains Rust source files used by integration tests:
- These files contain `literal!()` macros at known positions
- Used to test source file parsing and modification
- Should not be modified by tests (read-only test data)

## License

`jeb-literal` is Copyright Jeremy Banks, released under the familiar choice of `MIT OR Apache-2.0`.

This is heavily based on [the `expect-test` library](https://docs.rs/expect-test), which is also under `MIT OR Apache-2.0` and is Copyright the rust-analyzer developers, including Aleksey Kladov and Dylan MacKenzie.

## Related Work

- [expect-test](https://docs.rs/expect-test): Snapshot testing for Rust
- [databake](https://docs.rs/databake): Rust code generation for data
- [insta](https://docs.rs/insta): Another snapshot testing library
