# Litter Examples

These examples demonstrate how to use the `litter` library to create self-modifying programs.

## Running the Examples

All examples require the `LITTER_MODE=write` environment variable to actually update their source files.

### Counter Example

A simple counter that tracks how many times the program has been run:

```bash
cd examples
LITTER_MODE=write cargo run --example counter
```

Each time you run it, the counter in the source code increments!

### Configuration Example

Demonstrates self-updating configuration values:

```bash
LITTER_MODE=write cargo run --example config
```

The program adjusts its configuration and saves it back to the source file.

## How It Works

1. The `litter!()` macro creates a special value that knows its location in the source code
2. When you call `.set()` on a litter value, it updates both:
   - The in-memory value (immediately)
   - The source file (if `LITTER_MODE=write` is set)
3. The source file is parsed, modified using character-range splicing to preserve formatting

## Modes

- **Write** (`LITTER_MODE=write`): Changes to litter values are written back to source
- **Verify** (default in tests): Check that values round-trip correctly (for testing)
- **Memory** (`LITTER_MODE=memory`): Changes in memory only, no file writes
- **Reject** (`LITTER_MODE=reject`): Rejects any write attempts

## Use Cases

- Self-modifying scripts that track state between runs
- Configuration that evolves with usage
- Scripts that "learn" from execution
- Rust-script programs with embedded, persistent configuration
