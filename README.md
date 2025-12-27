# Litter

<img src="assets/litter.jpg" align="right" alt="a drawing showing papers littered on the ground, dirty and crumpled, with indistinct writing" height="196" />

`litter` provides mutable literals as smart pointers into your source code.

## Overview

Litter lets you create values that can update themselves in your source code. It's like snapshot testing, but for any data in your program.

```rust
use litter::litter;

fn main() {
    let mut counter = litter!(0u32);

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
- **Format-Preserving**: Uses prettyplease for consistent formatting

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
litter = "0.20220713"
```

Create a litter value:

```rust
use litter::litter;

let mut value = litter!(42u32);
```

Update it:

```rust
value.set(100u32);
// In UPDATE mode, this writes to your source file!
```

## Modes

Litter has three modes, controlled by environment variables:

- **Inactive** (default): Normal behavior, no file updates
- **Update** (`LITTER_UPDATE=1`): Changes are written back to source files
- **Verify** (`LITTER_VERIFY=1`): Validates that values round-trip correctly

```bash
LITTER_UPDATE=1 cargo run
```

## How It Works

1. The `litter!()` macro captures the source location (file, line, column)
2. Values implement the `Bake` trait from [databake](https://docs.rs/databake) for serialization
3. When `.set()` is called, litter:
   - Updates the in-memory value
   - Parses the source file
   - Finds the macro at the recorded location
   - Replaces its content with the baked new value
   - Formats and writes the file back

## Supported Types

Any type implementing `Bake + PartialEq + Clone` works with litter:

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

`litter` is Copyright Jeremy Banks, released under the familiar choice of `MIT OR Apache-2.0`.

This is heavily based on [the `expect-test` library](https://docs.rs/expect-test), which is also under `MIT OR Apache-2.0` and is Copyright the rust-analyzer developers, including Aleksey Kladov and Dylan MacKenzie.

## Related Work

- [expect-test](https://docs.rs/expect-test): Snapshot testing for Rust
- [databake](https://docs.rs/databake): Rust code generation for data
- [insta](https://docs.rs/insta): Another snapshot testing library
