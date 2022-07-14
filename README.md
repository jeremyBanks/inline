# 🗑️ `litter`

`litter` provides mutable literals as smart pointers into your source code.

These can be used for snapshot testing, or as a basic way of inlining state into
scripts. This is only intended for use in code that's being run through Cargo,
as it relies on `CARGO_` environment variables to locate the source code. The
implementation uses the `#[track_caller]` attribute (no macros).

## Core Capability

The `litter::Litter` type wraps a literal value with information about its
location in your source code, allowing it to be mutated with changes reflected
in the original script file. Literal types supported are integers (`1`, `2`,
`2_u8`, `-1i16`), floats (`1.5`, `2e6f64`), booleans (`true`, `false`), static
strings (`"hello"`, `r##"world##"`), static byte strings (`b"one two \x12"`,
`br"hell\x00"`).

`litter::Litter::inline(literal)` is also available as
`litter::inline(literal)`, or as the extension method
`litter::LiteralExt::edit(self)` on supported types. They're equivalent.

Dereferencing a `Litter` produces a `LitterHandle`. It implements `Deref` and
`DerefMut` exposing the inner value, as well as various other traits. If the
inner value is modified, it will be written back to the file when the `Litter`
is dropped.

Here's a basic example, of a string that's modified each time the script runs:

```rust
use litter::LiteralExt;

fn main() {
    let mut p = "".edit();
    *p += "hello world";
}
```

```rust
let mut p = "hello world".edit();
*p += "hello world";
```

```rust
let mut p = "hello worldhello world".edit();
*p += "hello world";
```

## Composition

The `Litter` constructors work using `#[track_caller]`, not macros, so it's
possible to wrap them to create your own functions that modifying literals, with
one important caveat: the literal in question must be the first literal that
occurs as an argument in the function call. So `f(x, "literal")` works, but
`f(2, "literal")` would not. For this reason we usually prefer to take the
literal as the first argument to the function.

For example, we can use this to implement snapshot testing in the style of
`expect_test`.

```rust
#[test]
fn test_the_ultimate_question() {
    assert_eq_u64(42, 6 * 9);
}

#[track_caller] // <- in order to look for literal at this function's call site instead
fn assert_eq_u64(expected: u64, actual: u64) {
    let expected = litter::inline(expected);

    if expected != actual {
        if std::env::get("UPDATE_EXPECT").unwrap_or("0") != "0" {
            *expected = actual; // <- updated in memory, written to source file at end of scope
        } else {
            panic!("\
                Expected {expected:?} but actual value was {actual:?}.\n\

                To update the expected value to {actual:?}, run this again with UPDATE_EXPECT=1.\
            ");
        }
    }
}
```

Running this test will initially fail with our panic message, but running it
again with `UPDATE_EXPECT=1 cargo test` will pass and update our source code to
reflect the correct value:

```rust
#[test]
fn test_the_ultimate_question() {
    assert_eq_u64(54, 6 * 9);
}
```

Although you don't need to write this particular function yourself: a generic
version is included at `litter::assert_eq(literal, actual)`.

## `serde` Support

If you enable the `json`, `yaml`, `postcard`, or `toml` features, the `.edit()`
handles for strings and bytes gain a corresponding method that can be used to
inline non-primitive values that implement `serde::{ Serialize, Deserialize }`.

```rust
fn main() {
    let list = "[]".edit().json::<Vec<u8>>();
    list.push(list.len() + 1);
}
```

```rust
let list = "[1]".edit().json::<Vec<u8>>();
list.push(list.len() + 1);
```

```rust
let list = "[1, 2]".edit().json::<Vec<u8>>();
list.push(list.len() + 1);
```

## External Values

If you would prefer to store the literal values in external values (alongside
your source code), rather than inline in the source code itself, you can use
`litter::Litter::external("some_relative_file_path.txt")` (also available as
`litter::external`).

## License

`litter` is Copyright Jeremy Banks, released under the familiar choice of
`MIT OR Apache-2.0`.

`litter` copies heavily from the
[the `expect-test` library](https://docs.rs/expect-test), which is also under
`MIT OR Apache-2.0` and is Copyright the rust-analyzer developers, including
Aleksey Kladov and Dylan MacKenzie.
