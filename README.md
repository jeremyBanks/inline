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

Update it (two syntaxes):

```rust
// Option 1: DerefMut syntax (most ergonomic)
*value += 1;  // Automatically writes when dropped

// Option 2: Direct field assignment
value.value = 100u32;

// Both write to your source file in WRITE mode!
```

## Modes

jeb-literal has four modes, controlled by the `LITERAL_MODE` environment variable:

- **Write** (default outside tests): Changes are written back to source files
- **Verify** (default in tests): Validates that values round-trip correctly
- **Memory** (`LITERAL_MODE=memory`): Changes in memory only, no file writes
- **Reject** (`LITERAL_MODE=reject`): Rejects any write attempts, always fails

```bash
LITERAL_MODE=write cargo run          # Enable self-modifying mode
LITERAL_MODE=memory cargo run         # Run without file writes
cargo test                            # Verify mode (default in tests)
LITERAL_MODE=write cargo test         # Update all snapshots
```

## How It Works

1. The `literal!()` macro captures the source location (file, line, column)
2. Values implement the `Bake` trait from [databake](https://docs.rs/databake) for serialization
3. The registry uses **index-based keys** (Nth literal in file) for stability across line insertions
4. When mutated (via `.value =` field or `DerefMut`), jeb-literal:
   - Detects the change (by comparing baked tokens)
   - Parses the source file
   - Finds the macro by its stable index
   - Uses character-range splicing to replace only the macro's value
   - Writes the file back (original formatting is preserved)

## Supported Types

Any type implementing `Bake + Clone` works with jeb-literal:

- Primitives: `u32`, `i64`, `f32`, `bool`, etc.
- Strings: `String`, `&str`
- Collections: `Vec<T>`, arrays, tuples
- And more via databake's built-in implementations

Note: `Clone` is required for write-on-drop functionality. Values are compared by their baked representation (tokens), not by `PartialEq`.

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

## Testing

Tests are organized into two categories:

- **Parallel-safe tests**: `concurrent_process_detection`, `multi_threaded` (don't use environment variables)
- **Serial tests**: All tests with `_serial` in the filename (use environment variables)

Run all tests serially (recommended):

```bash
cargo test -- --test-threads=1
```

Run only parallel-safe tests:

```bash
cargo test --test concurrent_process_detection --test multi_threaded
```

Run a specific serial test:

```bash
cargo test --test integration_serial_test -- --test-threads=1
```

## License

`jeb-literal` is Copyright Jeremy Banks, released under the familiar choice of `MIT OR Apache-2.0`.

This is heavily based on [the `expect-test` library](https://docs.rs/expect-test), which is also under `MIT OR Apache-2.0` and is Copyright the rust-analyzer developers, including Aleksey Kladov and Dylan MacKenzie.

## Related Work

- [expect-test](https://docs.rs/expect-test): Snapshot testing for Rust
- [databake](https://docs.rs/databake): Rust code generation for data
- [insta](https://docs.rs/insta): Another snapshot testing library
