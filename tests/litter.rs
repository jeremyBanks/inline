use litter::{LiteralExt, Litter};

#[test]
fn read_litter() {
    let lit = Litter::new("hello!");

    assert_eq!("hello!", lit.read());
}

#[test]
fn read_literal() {
    let lit = "hello!".litter();

    assert_eq!("hello!", lit.read());
}

#[test]
fn write_litter() {
    let lit = Litter::new("hello!");

    assert_eq!("hello!", lit.read());
    lit.write("goodbye!");
    assert_eq!("goodbye!", lit.read());
    lit.write("hello!");
    assert_eq!("hello!", lit.read());
}

#[test]
fn write_literal() {
    let lit = "hello!".litter();

    assert_eq!("hello!", lit.read());
    lit.write("goodbye!");
    assert_eq!("goodbye!", lit.read());
    lit.write("hello!");
    assert_eq!("hello!", lit.read());
}
