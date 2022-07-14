# litter

`litter` provides mutable literals as smart pointers into your source code.

## Example 1: `.edit()`

The `::litter::PrimitiveExt::edit` method can be used to create a source-editing
smart point wrapper of a literal value. In this example we can see how a string
literal could change as we repeatedly run a script.

```rust
use ::litter::PrimitiveExt;

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

## Example 2: serde

If you enable the `json`, `yaml`, `postcard`, or `toml` features, the `.edit()` handles
gain a corresponding method that can be used to inline non-primitive values that implement
`serde::{ Serialize, Deserialize }`.

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

## license

`litter` is Copyright Jeremy Banks, released under the familiar choice of `MIT OR Apache-2.0`.

`litter` is significantly based on [the `expect-test` library](https://docs.rs/expect-test),
which is also under `MIT OR Apache-2.0` and is Copyright the rust-analyzer developers,
including Aleksey Kladov and Dylan MacKenzie.
