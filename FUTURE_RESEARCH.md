# Future Research Topics for Litter

## Cargo Script Compatibility

Investigate compatibility with `cargo-script` / `rust-script` for single-file Rust programs with shebang support.

**Goal**: Self-modifying single-file Rust scripts that can update their own embedded configuration values.

**Questions**:
- Does cargo-script set the same env vars as `cargo run`? (CARGO, CARGO_MANIFEST_DIR, etc.)
- Can we detect cargo-script execution separately?
- Do file paths work correctly in cargo-script context?
- How do we handle source file location when script is run from different directories?

**Use case**:
```rust
#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! litter = "0.20220713"
//! ```

fn main() {
    let config = litter!(42);
    // Script modifies its own config value
    config.set(calculate_new_value());
}
```

## Other Topics

- **Performance**: Benchmark AST parsing/traversal overhead for large files
- **Error Recovery**: Better error messages when macros can't be found
- **Multi-file Support**: Track litter values across module boundaries
- **IDE Integration**: LSP support for showing current vs. source values
- **Serialization Formats**: Support JSON/TOML/YAML in addition to Rust syntax
- **Diff Visualization**: Show what changed when updating snapshots
- **Parallel Safety**: Better thread-local state management for parallel tests
