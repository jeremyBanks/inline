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

---

