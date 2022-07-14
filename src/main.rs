use bstr::ByteSlice;
use postcard;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Foo {
    name: String,
    id: usize,
}

fn main() {
    let foo = Foo {
        name: "John".to_string(),
        id: 1,
    };
    let args = std::env::vars().collect::<Vec<_>>();
    let bytes = postcard::to_stdvec(&args).unwrap();
    println!("{}", byte_literal(&bytes, 0));

    panic!("you can't use a formatting that's not rustfmt-friendly, eh?");
    // might be good enough to get started, though
}

fn byte_literal(bytes: &[u8], indent: usize) -> String {
    let mut s = String::with_capacity((bytes.len() + 1) * 4);
    let mut last_newline_index = 2;
    let mut previous_was_printable = false;
    write!(s, "b\"").unwrap();
    for byte in bytes {
        let line_length = s.len() as i64 - last_newline_index;
        let line_broken = line_length + 4 > 64;
        if line_broken {
            write!(s, "\\\n  {}", " ".repeat(indent)).unwrap();
            last_newline_index = s.len() as i64;
        }
        match byte {
            b'\n' => {
                // a newline following a printable character is converted into an escaped
                // newline to maintain the content and our own indentation, using \n.
                // other newlines are hex-encoded like a non-printable value, using \x10.
                if previous_was_printable {
                    write!(s, "\\n\\\n  {}", " ".repeat(indent)).unwrap();
                    last_newline_index = s.len() as i64;
                } else {
                    write!(s, "\\x0A").unwrap();
                }
                previous_was_printable = false;
            }
            // leading spaces (following a backslashed-newline) would be trimmed unless encoded
            b' ' => {
                if line_broken || !previous_was_printable {
                    write!(s, "\\x20").unwrap();
                } else {
                    write!(s, " ").unwrap();
                }
                previous_was_printable = true;
            }
            // most ascii printable characters are left as-is
            b'!' | b'#'..=b'[' | b']'..=b'~' => {
                write!(s, "{}", *byte as char).unwrap();
                previous_was_printable = true;
            }
            // but backslash and double quotes are uppercase hex escaped along with everything else
            b'\\' | b'"' | 0..=0x1F | 0x7F..=0xFF => {
                write!(s, "\\x{byte:02X}").unwrap();
                previous_was_printable = false;
            }
        }
    }
    s.push_str("\"");
    s
}
