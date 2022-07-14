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
    let bytes = postcard::to_stdvec(&foo).unwrap();
    println!("{}", byte_literal(&bytes, 0));

    let bytes = (0..=0xFF).into_iter().collect::<Vec<u8>>();
    println!("{}", byte_literal(&bytes, 0));

    let x = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x10\x11\x12\
                          \x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F !\x22#$%&'()*+,-./0123\
                          456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\x5C]^_`abcdefghijklmnopqrstuvwxyz{\
                          |}~\x7F\x80\x81\x82\x83\x84\x85\x86\x87\x88\x89\x8A\x8B\x8C\x8D\x8E\x8F\x90\
                          \x91\x92\x93\x94\x95\x96\x97\x98\x99\x9A\x9B\x9C\x9D\x9E\x9F\xA0\xA1\xA2\xA3\
                          \xA4\xA5\xA6\xA7\xA8\xA9\xAA\xAB\xAC\xAD\xAE\xAF\xB0\xB1\xB2\xB3\xB4\xB5\xB6\
                          \x20       \xB7\xB8\xB9\xBA\xBB\xBC\xBD\xBE\xBF\xC0\xC1\xC2\xC3\xC4\xC5\xC6\xC7\xC8\xC9\
                          \xCA\xCB\xCC\xCD\xCE\xCF\xD0\xD1\xD2\xD3\xD4\xD5\xD6\xD7\xD8\xD9\xDA\xDB\xDC\
                          \xDD\xDE\xDF\xE0\xE1\xE2\xE3\xE4\xE5\xE6\xE7\xE8\xE9\xEA\xEB\xEC\xED\xEE\xEF\
                          \xF0\xF1\xF2\xF3\xF4\xF5\xF6\xF7\xF8\xF9\xFA\xFB\xFC\xFD\xFE\xFF";

    println!("{}", byte_literal(x, 0));
    println!("  {}", byte_literal(x, 2));
    println!("    {}", byte_literal(x, 4));

    let x = (0..=0xFF).into_iter().map(|_| 0).collect::<Vec<u8>>();

    println!("{}", byte_literal(&*x, 0));
    println!("  {}", byte_literal(&*x, 2));
    println!("    {}", byte_literal(&*x, 4));
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
                    write!(s, "\\x10").unwrap();
                }
                previous_was_printable = false;
            }
            // leading spaces (following a backslashed-newline) would be trimmed unless encoded
            b' ' => {
                if line_broken {
                    write!(s, "\\x20").unwrap();
                } else {
                    write!(s, " ").unwrap();
                }
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
