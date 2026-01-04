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

---

