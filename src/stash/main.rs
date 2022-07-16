use bstr::ByteSlice;
use postcard;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

fn main() {
    let args = (
        vec!["hello", "goodbye\n:("],
        2,
        3,
        "more like good-byte! ".repeat(100),
    );
    let bytes = postcard::to_stdvec(&args).unwrap();
    println!("{}", byte_literal(&bytes, 0));
}
