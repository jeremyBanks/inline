# `stringify_verbatim` Crate Proposal

A proc-macro crate that preserves original whitespace when converting tokens to a string literal.

## Problem

The built-in `stringify!` macro normalizes whitespace:

```rust
stringify!(foo   bar    baz)  // → "foo bar baz" (single spaces)
stringify!(a\n    b)          // → "a b" (newline + indent → space)
```

## Solution

A proc-macro that uses span information to reconstruct the original formatting:

```rust
use stringify_verbatim::stringify_raw;

stringify_raw!(foo   bar    baz)  // → "foo   bar    baz" (preserved!)
stringify_raw!(
    fn example() {
        42
    }
)  // → "fn example() {\n        42\n    }" (preserves newlines + indentation)
```

## Implementation Approach

1. Proc-macro receives `TokenStream` with span locations
2. For each token, `Span::start()` and `Span::end()` provide `LineColumn`
3. Compute whitespace between tokens from gaps in positions
4. Reconstruct original formatting:
   - Same line: `end_col` to `start_col` difference = spaces
   - Different line: insert newlines + indentation (from `start_col`)

## API

```rust
// Main macro - returns &'static str
stringify_raw!(any tokens here)

// Alternative: return String for runtime flexibility
stringify_raw_owned!(any tokens here)
```

## Use Case: `inline` crate

The `inline::tokens!` macro could use this to preserve original formatting:

```rust
// Current behavior (whitespace normalized):
let t = tokens!(foo   bar);  // stores "foo bar"

// With stringify_verbatim:
let t = tokens!(foo   bar);  // stores "foo   bar"
```

## Notes

- Requires `proc_macro` (not `macro_rules!`) to access real span info
- `span-locations` feature of `proc_macro2` enables this in proc-macro context
- Comments are NOT preserved (not part of token stream)
