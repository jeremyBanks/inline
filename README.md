# inline

Self-modifying values that update their source code at runtime.

## Overview

`inline` lets you create values that can update themselves in your source code. This is an experimental approach to snapshot testing and self-modifying code in Rust.

```rust
use inline::cell;

fn main() {
    let mut counter = cell(0u32);

    println!("Run #{}", *counter + 1);

    let current = *counter;
    counter.value = current + 1;
    // The source file is now updated with the new value!
}
```

## Features

- **Self-Modifying Code**: Values that update their own source code
- **Write-on-Drop**: Mutations through `DerefMut` automatically persist
- **Default Values**: `cell_default::<T>()` uses `Default::default()`
- **One-Shot Replacement**: `replace()` substitutes entire expressions
- **Type-Safe**: Uses Rust's type system and databake for serialization
- **Mode-Based**: Control when updates happen via environment variables
- **Stable Positions**: Index-based tracking survives line insertions
- **Thread-Safe**: File-level locking prevents corruption
- **Format-Preserving**: Character-range splicing preserves original formatting

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
inline = "0.0.1-dev.1"
```

### Mutable Cells

Create a mutable cell that persists changes to source code:

```rust
use inline::cell;

let mut value = cell(42u32);
let counter = cell_default::<u32>();  // Uses Default::default()

// Mutate via DerefMut (writes on drop)
*value += 1;

// Or direct field assignment
value.value = 100u32;

// Both write to your source file in Write mode!
```

Function aliases: `cell`, `var`, `snapshot`, `HACK`

Macro versions: `cell!()`, `cell_default!()`

### One-Shot Replacement

Replace the entire call expression with a baked value:

```rust
use inline::replace;

// First run: evaluates expression, replaces call with result
let author = replace(std::env::var("USER").unwrap_or_default());
// After first run, this becomes: let author = "jeremy";
```

Function aliases: `replace`, `val`, `eval`, `REPLACE_ME`

Macro versions: `replace!()`, `replace_default!()`

## Modes

`inline` has four modes, controlled by the `INLINE_MODE` environment variable:

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

### Cargo Subcommand

For convenience, install the `cargo inline-write` subcommand:

```bash
cargo install --path . --bin cargo-inline-write

# Now you can regenerate all test snapshots easily:
cargo inline-write
cargo inline-write -- --test-threads=1
```

This is equivalent to `INLINE_MODE=write cargo test` but easier to remember and type.

## How It Works

1. Functions capture the source location via `#[track_caller]`
2. Values implement the `Bake` trait from [databake](https://docs.rs/databake) for serialization
3. The registry uses **index-based keys** (Nth call in file) for stability across line insertions
4. When mutated, inline:
   - Detects the change (using `PartialEq`)
   - Parses the source file
   - Finds the call by its stable index
   - Uses character-range splicing to replace only the value
   - Writes the file back (original formatting is preserved)

### Replacement Behavior

- **Function calls**: the last argument is replaced (trailing position for extensibility)
- **Method calls**: the receiver expression is replaced
- **Macros**: entire contents inside delimiters are replaced

For `replace()` mode, the entire call/macro expression is replaced with the baked value.

## Supported Types

Any type implementing `Bake + Clone + PartialEq` works with inline:

- Primitives: `u32`, `i64`, `f32`, `bool`, etc.
- Strings: `String`, `&str`
- Collections: `Vec<T>`, arrays, tuples
- And more via databake's built-in implementations

Note: `Clone` is required for write-on-drop functionality. Values are compared using `PartialEq` to detect changes; `Bake` is only used for serialization.

### Extension Trait

The `InlineCellExt` trait provides additional methods without polluting the inner type's namespace:

```rust
use inline::{cell, InlineCellExt};

let mut x = cell(42);
x.value = 100;
x.flush()?;  // Write immediately, don't wait for drop
x.reset_to_default();  // Reset to type's default value
```

Methods: `flush()`, `reset_to_default()`, `path()`, `line()`, `column()`, `index()`.

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
- Call count remains stable (adding/removing calls changes indices)
- Only value content changes (the call's position in file remains the same)

## Testing

Tests are organized into two categories:

- **Parallel-safe tests**: `concurrent_process_detection`, `multi_threaded` (don't use environment variables)
- **Serial tests**: All tests with `_serial` in the filename (use environment variables)

Run all tests serially (recommended):

```bash
cargo test -- --test-threads=1
```

## License

`inline` is Copyright Jeremy Banks, released under the familiar choice of `MIT OR Apache-2.0`.

This is heavily based on [the `expect-test` library](https://docs.rs/expect-test), which is also under `MIT OR Apache-2.0` and is Copyright the rust-analyzer developers, including Aleksey Kladov and Dylan MacKenzie.

## Related Work

- [expect-test](https://docs.rs/expect-test): Snapshot testing for Rust
- [databake](https://docs.rs/databake): Rust code generation for data
- [insta](https://docs.rs/insta): Another snapshot testing library
