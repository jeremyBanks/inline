# Inline Examples

These examples demonstrate how to use the `inline` library to create self-modifying programs.

## Running the Examples

All examples require the `INLINE_MODE=write` environment variable to actually update their source files.

### Counter Example

A simple counter that tracks how many times the program has been run:

```bash
cd examples
INLINE_MODE=write cargo run --example counter
```

Each time you run it, the counter in the source code increments!

### Configuration Example

Demonstrates self-updating configuration values:

```bash
INLINE_MODE=write cargo run --example config
```

The program adjusts its configuration and saves it back to the source file.

## How It Works

1. The `cell()` function (or `cell!()` macro) creates a value that knows its location in the source code
2. When you modify `cell.value` and the cell is dropped, it updates both:
   - The in-memory value (immediately)
   - The source file (if `INLINE_MODE=write` is set)
3. The source file is parsed, modified using character-range splicing to preserve formatting

## Modes

- **Write** (`INLINE_MODE=write`): Changes to cell values are written back to source
- **Verify** (default in tests): Check that values match source (for snapshot testing)
- **Memory** (`INLINE_MODE=memory`): Changes in memory only, no file writes
- **Reject** (`INLINE_MODE=reject`): Rejects any write attempts

## Use Cases

- Snapshot testing with self-updating expected values
- Self-modifying scripts that track state between runs
- Configuration that evolves with usage
- Rust-script programs with embedded, persistent configuration
