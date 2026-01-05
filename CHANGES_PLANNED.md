# Planned Changes

This file tracks changes we plan to make to the implementation based on design review.

## From Design Review

### 1. Use PartialEq for Change Detection Instead of Token Comparison

**Current:** We compare `value.bake()` token streams to detect changes
**Planned:** Use `PartialEq` to compare values directly

**Rationale:** Comparing tokens is unnecessarily complex. Most types implement `PartialEq`, and it's more intuitive.

**Changes needed:**
- Add `PartialEq` bound to `Value` trait
- Update `Drop` implementation to use `self.original == self.literal`
- Update `LiteralInner::set()` to use `PartialEq`
- Still need `Bake` for serialization to source code

**Impact:** Simpler, more intuitive change detection

### 2. Default to Memory Mode When Not Running Under Cargo

**Current:** Write mode is default outside tests
**Planned:** Memory mode is default when not run by `cargo`, Write mode only when run by `cargo`

**Rationale:** Safety - don't try to modify source files when running a compiled binary directly

**Detection:** Check for cargo environment variables (`CARGO`, `CARGO_MANIFEST_DIR`, `CARGO_PKG_NAME`)

**Changes needed:**
- Update `Mode::default_for_context()` logic
- Check for cargo env vars before defaulting to Write

**Impact:** Safer defaults, compiled binaries won't attempt file modification

### 3. Add "write" Default Cargo Feature

**Current:** No feature flag to disable write functionality
**Planned:** Add a default cargo feature `"write"` that can be disabled to prevent all file writes

**Rationale:** Allow users to compile out write functionality entirely for production builds

**Changes needed:**
- Add `[features]` section to `Cargo.toml` with `default = ["write"]` and `write = []`
- Guard write code with `#[cfg(feature = "write")]`
- Make Write mode behave like Memory mode when feature is disabled

**Impact:** Users can use `default-features = false` to guarantee no file writes at compile time

### 4. Verify Initial Value on First Access

**Current:** We only verify values when they're written/modified
**Planned:** In Verify mode, also verify that the initial value provided to `literal!()` matches what's in the source file

**Rationale:** Catch cases where the code says `literal!(42)` but the file actually contains `literal!(100)`

**Changes needed:**
- In `get_or_create`, after resolving index, verify initial value matches source in Verify mode
- Only on first access (when creating new registry entry)
- Panic with helpful message if mismatch detected

**Impact:** Earlier detection of snapshot mismatches, catches stale hardcoded values in tests

### 5. Change Registry Key to Use (line, column) Instead of Index

**Current:** Registry key is `(PathBuf, usize, TypeId)` where `usize` is the stable index
**Planned:** Registry key is `(PathBuf, u32, u32, TypeId)` where the `u32`s are line and column

**Rationale:**
- Compile-time (line, column) from `file!()`, `line!()`, `column!()` never changes
- Eliminates file parsing on reads - just hash map lookup
- Stable index only needed when actually writing/verifying, resolved lazily

**Changes needed:**
- Update `RegistryKey` type definition
- Remove index resolution from `get_or_create`
- Lazy index resolution on first write/verify for a file
- Build and cache (line, column) → stable index mapping per file
- Use cached mapping for subsequent writes in the same file

**Impact:** No file I/O on reads, faster literal access

---

