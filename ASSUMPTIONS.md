# Implementation Assumptions and Decisions

This document lists all assumptions and design decisions made during the implementation of inline with databake integration.

## Core Architectural Decisions

### 1. No Global AST Caching

**Decision**: Parse, modify, and write files on each update instead of caching ASTs globally.

**Rationale**:
- `syn::File` contains `proc_macro::Span` which is not `Send`, making it impossible to share across threads safely in a static
- Simpler implementation without complex state management
- File system provides natural synchronization
- Acceptable performance for expected use case (infrequent updates)

**Impact**: Each `set()` call re-parses the entire source file

**Alternative Considered**: Using complex thread-local storage or unsafe code to work around Span limitations. Rejected for complexity and safety concerns.

### 2. Immediate Writes (No Batching)

**Decision**: Write to disk immediately on each `set()` call, no deferred flushing.

**Rationale**:
- Simple, predictable behavior
- No lost updates on program crash
- Matches user expectations (explicit `set()` = explicit write)
- No need for explicit `flush()` calls

**Impact**: More file I/O, but simpler API and safer behavior

**Alternative Considered**: Batch updates and flush on program exit. Rejected because crashes would lose data.

### 3. File-Level Locking

**Decision**: Use a global lock for all file updates (single file can only be updated by one thread at a time).

**Rationale**:
- Prevents file corruption from concurrent writes
- Simple implementation using parking_lot::Mutex
- Acceptable for expected use case (scripts, not high-concurrency servers)

**Current Limitation**: The lock is currently global (all files share one lock), not per-file.

**Impact**: Only one file can be updated at a time across the entire process

**Future Improvement**: Could use per-file locks with Arc<Mutex<()>> per PathBuf

### 4. Location-Based Macro Identification

**Decision**: Use (file, line, column) tuple to identify which macro to update.

**Assumption**: Line and column numbers remain stable because we only modify the value inside the macro, not the structure.

**Risk**: If prettyplease significantly reformats code, positions could shift.

**Mitigation**: prettyplease is deterministic and only changes the macro's contents, not its position.

**Verified By**: Integration tests confirm positions remain stable across updates.

### 5. Mode Checking via Environment Variables

**Decision**: Check environment variables on every `set()` call instead of caching at startup.

**Rationale**:
- Tests can change environment variables mid-execution
- More flexible for dynamic behavior
- Minimal performance cost (env var lookup is fast)

**Impact**: Tests can set INLINE_MODE=write and see immediate effect

**Alternative Considered**: Cache mode in a Lazy static. Rejected because it wouldn't work with tests that set env vars after startup.

## Type System Decisions

### 6. Using databake's Bake Trait

**Decision**: Use databake's existing `Bake` trait instead of implementing custom serialization.

**Rationale**:
- databake already handles many Rust types
- Type-safe, compile-time validated serialization
- Well-tested and maintained

**Limitation**: Only types implementing `Bake` can be used with inline

**Current Support**: Primitives, Vec, tuples, and other std types via databake's built-in implementations

**Future**: Users can implement `Bake` for custom types

### 7. Literal Trait Requirements

**Decision**: Require `Bake + PartialEq + Clone`

**Rationale**:
- `Bake`: Needed for serialization to Rust code
- `PartialEq`: Needed to detect when value changes (optimization: skip update if unchanged)
- `Clone`: Needed for rollback on write failure

**Impact**: Some types may not be usable if they don't implement these traits

### 8. databake Output Format

**Assumption**: Accept whatever databake produces, even if verbose (e.g., `alloc::vec::Vec::<u32>`).

**Rationale**:
- databake's output is correct and type-safe
- prettyplease will format it consistently
- Self-modifying code doesn't need to be human-readable
- Can be improved later if needed

**Example**: `vec![1, 2, 3]` might be baked as more verbose form, but it's still valid Rust

## Parsing and Formatting Decisions

### 9. syn span-locations Feature Required

**Assumption**: `proc-macro2` with `span-locations` feature provides accurate line/column info.

**Verified**: Integration tests confirm this works correctly.

**Risk**: If syn changes span behavior in future versions, location tracking could break.

**Mitigation**: Tests will catch this.

### 10. Character-Range Splicing for Formatting Preservation

**Decision**: Use character-range splicing to replace only the macro value, preserving all original formatting.

**Impact**:
- Original formatting is preserved
- Only the value inside the macro changes
- Indentation, spacing, comments, etc. remain unchanged

**Rationale**:
- Minimizes diff noise in version control
- Preserves user's formatting preferences
- More intuitive for self-modifying code
- Works reliably with proc-macro2 span locations

**Implementation**: Convert line/column positions to byte offsets, then splice the new value into the exact character range of the old value.

**Previous Approach**: Originally used prettyplease to reformat entire files. Changed to character-range splicing to preserve formatting.

### 11. Qualified Path Handling

**Decision**: Support both `inline!()` and `inline::inline!()` by checking the last path segment.

**Implementation**: In `MacroReplacer`, use `mac.path.segments.last()` instead of `get_ident()`.

**Rationale**:
- Users might import the macro or use it qualified
- More flexible and robust
- Same approach used in test helpers

**Bug Fixed**: Initial implementation only worked with unqualified paths.

## Testing Decisions

### 12. Serial Test Execution Required

**Limitation**: Tests must run with `--test-threads=1` due to environment variable conflicts.

**Cause**: Multiple tests set `INLINE_MODE` env var, which affects other tests running in parallel.

**Rationale**: Using env vars is simpler than implementing a thread-safe mode override mechanism.

**Impact**: Tests run slower, but are more reliable.

**Alternative Considered**: Add `#[serial]` attribute (requires serial_test crate). Decided to keep dependencies minimal.

### 13. Test File Isolation

**Decision**: Each test creates temporary files with `tempfile::TempDir`.

**Rationale**:
- Tests don't interfere with each other
- Automatic cleanup
- Can run tests in any order (when serial)

**Impact**: Tests are fully isolated and reproducible.

## Error Handling Decisions

### 14. Rollback on Failure

**Decision**: If file write fails, rollback the in-memory value to the previous state.

**Implementation**: Clone old value before updating, restore on error.

**Rationale**:
- Keeps in-memory state consistent with file state
- Prevents silent data loss
- User gets a warning but program continues

**Output**: Errors are printed to stderr but don't panic.

### 15. Non-Fatal Errors

**Decision**: Inline errors are warnings, not panics.

**Rationale**:
- File update failures shouldn't crash the program
- User code can continue running even if persistence fails
- Matches "best effort" philosophy for self-modifying code

**Impact**: Program continues even if inline can't update files (e.g., read-only filesystem).

## Future Considerations

### Not Yet Implemented

1. **Custom Bake Implementations**: Users can add these, but no examples yet
2. **External File Support**: Original `External` type not implemented with databake

### Intentionally Omitted

1. **Git Integration**: Out of scope (user's responsibility)
2. **Backup/History**: Out of scope
3. **Atomic Writes**: Not needed for current use case
4. **Watch Mode**: Complexity not justified
5. **Remote Files**: Only local filesystem supported

## Summary of Key Assumptions

1. **Span locations are accurate** (verified by tests)
2. **Line numbers remain stable** (only value changes, not structure)
3. **File system is writable** (panics on write failures in Write mode)
4. **Single process per file** (concurrent modification detection via read-verify-write)
5. **Tests run serially** (environment variable conflicts)
6. **databake output is acceptable** (even if verbose)
7. **Character-range splicing preserves formatting** (original formatting maintained)
8. **Immediate writes are acceptable** (no performance issues)
9. **Thread-local AST caching improves performance** (with Arc<RwLock<String>> for shared state)
10. **Type bounds are acceptable** (`Bake + PartialEq + Clone`)

All assumptions have been tested and documented. The implementation is complete and functional for the target use case: self-modifying Rust scripts with embedded, persistent configuration.
