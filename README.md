# Inline

<img src="assets/inline.jpg" align="right" alt="a drawing showing papers littered on the ground, dirty and crumpled, with indistinct writing" height="196" />

`inline` provides mutable literals as smart pointers into your source code.

## Overview

Inline lets you create values that can update themselves in your source code. It's like snapshot testing, but for any data in your program.

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
   - Replaces its content with the baked new value
   - Formats and writes the file back

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

- **Self-Executing Scripts**: Rust scripts that remember state between runs
- **Learning Programs**: Code that adapts based on execution history
- **Dynamic Configuration**: Config that evolves with usage
- **Development Tools**: Scripts that track their own usage patterns

## Implementation Details

### Architecture

- Global per-file locking prevents concurrent modification
- Immediate writes (no batching or delayed flushing)
- Uses `syn` for parsing, `prettyplease` for formatting
- Requires `proc-macro2` with `span-locations` feature for position tracking

### Assumptions

- Single process modifies each file (no external editors while updating)
- Source files are valid Rust that can be parsed
- Line/column positions remain stable (only value content changes)

## Testing

Run tests with:

```bash
cargo test -- --test-threads=1
```

Note: Tests must run serially due to shared environment variables.

## License

`inline` is Copyright Jeremy Banks, released under the familiar choice of `MIT OR Apache-2.0`.

This is heavily based on [the `expect-test` library](https://docs.rs/expect-test), which is also under `MIT OR Apache-2.0` and is Copyright the rust-analyzer developers, including Aleksey Kladov and Dylan MacKenzie.

## Related Work

- [expect-test](https://docs.rs/expect-test): Snapshot testing for Rust
- [databake](https://docs.rs/databake): Rust code generation for data
- [insta](https://docs.rs/insta): Another snapshot testing library
