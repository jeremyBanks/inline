`litter` makes your literals mutable with smart pointers into your source code.

These can be used for snapshot testing, or as a basic way of inlining state into
scripts. This is only intended for use in code that's being run through Cargo,
as it relies on `CARGO_` environment variables to locate the source code when
mutations need to be written back. The implementation uses the
[`#[track_caller]`](https://doc.rust-lang.org/reference/attributes/codegen.html#the-track_caller-attribute)
attribute (no macros).

## To Do

- [ ] implement naive unsyncronized writing
- [ ] implement better writing
- [ ] support external data(?)
- Future work?! Almost nothing described above has actually been implemented
  yet!
- Fallible alternatives instead of panicking, including for the case of writing
  values when the source files don't exist (they can still be saved in-memory).
- External data, not just inline.
- While the value has only been read, we can hold on to a Weak Arc of its state
  and let it be freed. Once it's been written to, we need to keep it in memory
  forever, so we leak a reference.

## Basic Use

The [`litter::Litter`] type wraps a literal value with information about its
location in your source code, allowing it to be mutated with changes reflected
in the original script file. Literal types supported are integers (`1`, `2`,
`2_usize`, `-1i16`), floats (`1.5`, `2e6f64`), booleans (`true`, `false`),
static strings (`"hello"`, `r##"world##"`), static byte strings
(`b"one two \x12"`, `br"hell\x00"`). These are described by the
[`litter::Literal`] trait.

`.edit()`ing a `Literal` or a `Litter` produces a `LitterHandle`. It implements
`Deref` and `DerefMut`, exposing the inner value, as well as various other
traits. If the inner value is modified, it will be written back to the file when
the `Litter` is dropped.

Here's a basic example, of a string that's modified each time the script runs:

```rust
use litter::LiteralExt;

fn main() {
    let mut p = "and I say hello!".edit();
    *p += " hello!";
}
```

```rust
    let mut p = "and I say hello! hello!".edit();
    *p += " hello!";
```

```rust
    let mut p = "and I say hello! hello! hello!".edit();
    *p += " hello!";
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
    let expected = litter::new(expected);

    if expected != actual {
        if std::env::get("UPDATE_EXPECT").unwrap_or("0") != "0" {
            *expected = actual; // <- updated in memory, written to source at end of scope
        } else {
            panic!("\
                Expected {expected:?} but actual value was {actual:?}.\n\
                \n\
                To update the expected value, run this again with UPDATE_EXPECT=1.\
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

## Serialization

If you enable the `json`, `yaml`, `postcard`, or `toml` features, `LiteralExt`
for strings and bytes gains corresponding `.edit_json()`, `.edit_yaml()`,
`.edit_toml()`, or `.edit_postcard()` methods that can be used to be used to
inline non-primitive values that implement `serde::{ Serialize, Deserialize }`,
as well as `Debug`, `Clone`, and `Default`. (If your type doesn't implement
`Default`, you may consider wrapping it in an `Option<...>`.)

As a special case for convenience, if the literal string is empty but
deserialization fails, the type's `Default::default()` value will be returned.
Any other (de)serialization errors will cause the program to panic.

```rust
fn main() {
    // empty string interpreted as default, like "[]"
    let json_vec = "".edit_json::<Vec<usize>>();
    json_vec.push(json_vec.len());

    // empty byte string interpreted as default, like b"\x00"
    let postcard_vec = b"".edit_postcard::<Vec<usize>>();
    postcard_vec.push(postcard_vec.len());
}
```

```rust
    let json_vec = "[0]".edit_json::<Vec<usize>>();
    json_vec.push(json_vec.len());

    let postcard_vec = b"\x01\x00".edit_postcard::<Vec<usize>>();
    postcard_vec.postcard_vec(postcard_vec.len());
```

```rust
    let json_vec = "[0, 1]".edit_json::<Vec<usize>>();
    json_vec.push(json_vec.len());

    let postcard_vec = b"\x02\x00\x01".edit_postcard::<Vec<usize>>();
    postcard_vec.push(postcard_vec.len());
```

If no type is specified or can be inferred, the `.edit_json()`, `.edit_yaml()`,
and `.edit_toml()` methods default to dynamic values (`serde_json::Value`,
`serde_yaml::Value`, and `toml::Value`). However `.edit_postcard()` always
requires a known type (it's non-self-describing and can't be handled
dynamically).

For text formats, encoding to a string literal will be pretty/verbose, while
encoding to a byte string literal will be use a compact representation. (This
obviously isn't relevant to binary-only formats like `postcard` and `rkyv`.)

## Performance and Reliability

This is (clearly) intended for convenience, not performance. It should be fast
enough for use in test snapshotting or dumping some state for a script, but you
certainly wouldn't want to use it for a high-throughput web server. Access to
each literal is controlled by an `RwLock` which may block if used concurrently.

The filesystem is only accessed when a mutated value needs to be written back,
so if a value is never then modified the filesystem won't be accessed, and in
that case the program can work fine on a different system without the source
files available.

Logic errors can occur if multiple copies of your program are running
concurrently and both try to modify the same file.

## License

`litter` is Copyright Jeremy Banks, released under the familiar choice of
`MIT OR Apache-2.0`.

`litter` copies heavily from the
[the `expect-test` library](https://docs.rs/expect-test), which is also under
`MIT OR Apache-2.0` and is Copyright the rust-analyzer developers, including
Aleksey Kladov and Dylan MacKenzie.

<!--
NB: We need to specify these manually (as links to docs.rs) so that they'll be
visible when README.md is rendered directly, such as on GitHub and on Crates.io.
However, when we're rendering it in the generate crate documentation, we want
to allow rustdoc to resolve the links for us, particularly for local types.
Therefore, any change made to these links below must be accompanied by a
corresponding change to `src/pre-readme.md`, which will take precedence and
allow us to use rustdoc's resolution.
-->

[`litter::Litter`]: https://docs.rs/litter/latest/litter/struct.Litter.html
[`litter::Literal`]: https://docs.rs/litter/latest/litter/struct.Literal.html
