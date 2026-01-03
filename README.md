# Inline

`inline` provides mutable literals as smart pointers into your source code.

## Overview

Inline lets you create values that can update themselves in your source code. This is an experimental approach to snapshot testing and self-modifying code in Rust.

```rust
use inline::inline;

fn main() {
    let mut counter = inline!(0u32);

    println!("Run #{}", *counter + 1);

    counter.set(*counter + 1);
    // The source file is now updated with the new value!
}
```

## Features

- **Self-Modifying Code**: Values that update their own source code
- **Type-Safe**: Uses Rust's type system and databake for serialization
- **Mode-Based**: Control when updates happen via environment variables
- **Thread-Safe**: File-level locking prevents corruption
- **Format-Preserving**: Character-range splicing preserves original formatting

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
inline = "0.20220713"
```

Create a inline value:

```rust
use inline::inline;

let mut value = inline!(42u32);
```

Update it:

```rust
value.set(100u32);
// In WRITE mode, this writes to your source file!
```

## Modes

Inline has four modes, controlled by the `INLINE_MODE` environment variable:

- **Write** (default outside tests): Changes are written back to source files
- **Verify** (default in tests): Validates that values round-trip correctly
- **Memory** (`INLINE_MODE=memory`): Changes in memory only, no file writes
- **Reject** (`INLINE_MODE=reject`): Rejects any write attempts, always fails

```bash
INLINE_MODE=write cargo run          # Enable self-modifying mode
INLINE_MODE=memory cargo run         # Run without file writes
cargo test                            # Verify mode (default in tests)
INLINE_MODE=write cargo test         # Update all snapshots
```

## How It Works

1. The `inline!()` macro captures the source location (file, line, column)
2. Values implement the `Bake` trait from [databake](https://docs.rs/databake) for serialization
3. When `.set()` is called, inline:
   - Updates the in-memory value
   - Parses the source file
   - Finds the macro at the recorded location
   - Uses character-range splicing to replace only the macro's value
   - Writes the file back (original formatting is preserved)

## Supported Types

Any type implementing `Bake + PartialEq + Clone` works with inline:

- Primitives: `u32`, `i64`, `f32`, `bool`, etc.
- Strings: `&str`
- Collections: `Vec<T>`, arrays, tuples
- And more via databake's built-in implementations

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
- Line/column positions remain stable (only value content changes)

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

`inline` is Copyright Jeremy Banks, released under the familiar choice of `MIT OR Apache-2.0`.

This is heavily based on [the `expect-test` library](https://docs.rs/expect-test), which is also under `MIT OR Apache-2.0` and is Copyright the rust-analyzer developers, including Aleksey Kladov and Dylan MacKenzie.

## Related Work

- [expect-test](https://docs.rs/expect-test): Snapshot testing for Rust
- [databake](https://docs.rs/databake): Rust code generation for data
- [insta](https://docs.rs/insta): Another snapshot testing library
