# Testing Guide

## Running Tests

### Serial Mode (Recommended)

Most tests should be run in serial mode to avoid conflicts when modifying source files:

```bash
cargo test -- --test-threads=1
```

### Parallel Mode

Some tests may fail in parallel mode due to concurrent file modifications:

```bash
cargo test
```

**Known Issues in Parallel Mode:**
- `static_persistence_serial_test::test_static_persistence_same_value` may fail due to file modification conflicts
- Tests that modify the same source files concurrently can interfere with each other

## Test Categories

### Serial Tests (Suffix: `_serial_test.rs`)

These tests modify source files and should be run serially:
- `concurrent_process_detection.rs` - Detects external file modifications
- `default_values_serial_test.rs` - Tests default value behavior
- `deref_mut_serial_test.rs` - Tests mutable dereference
- `index_stability_serial_test.rs` - **Critical**: Tests value persistence across line insertions
- `integration_serial_test.rs` - Integration tests
- `lazy_loading_serial_test.rs` - Tests behavior with non-existent files
- `line_number_stability_serial_test.rs` - Line number handling
- `line_wrapping_serial_test.rs` - Multi-line value formatting
- `multiline_value_serial_test.rs` - Multi-line value tests
- `position_stability_serial_test.rs` - Position tracking
- `real_source_modification_serial.rs` - Real file modification tests (some ignored)
- `simple_debug_serial_test.rs` - Debug output tests
- `static_persistence_serial_test.rs` - Registry persistence tests
- `verify_mode_serial_test.rs` - Verify mode behavior

### Parallel-Safe Tests

- `multi_threaded.rs` - Thread safety tests

## Test Results

As of the latest commit, all tests pass in serial mode:

```bash
$ cargo test -- --test-threads=1
...
test result: ok. 49 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out
```

## Design Changes Implemented

✅ **Completed:**
1. PartialEq-based change detection (replacing token comparison)
2. Hybrid registry key (stable index when file exists, (line, column) fallback)
3. Cargo detection for mode defaults
4. "write" feature flag (default-enabled, can be disabled)
5. Index stability - values persist across line insertions

⚠️ **Disabled:**
- Initial value verification (disabled due to false positives with databake formatting differences)

## Critical Tests

**Index Stability** - These tests verify the core behavior that values persist when lines are inserted above them:
```bash
cargo test --test index_stability_serial_test -- --test-threads=1
```

All 3 tests should pass:
- `test_index_resolution_is_consistent` - Index resolution remains stable
- `test_value_persists_across_line_insertions` - Values persist when lines inserted above
- `test_multiple_literals_maintain_distinct_identities` - Each literal maintains unique identity

**Lazy Loading** - Tests behavior with non-existent files:
```bash
cargo test --test lazy_loading_serial_test -- --test-threads=1
```

All 4 tests should pass, verifying graceful handling of missing source files.
