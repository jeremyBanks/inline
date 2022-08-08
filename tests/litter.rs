use inline::{LiteralExt, Litter};

#[test]
fn read_inline() {
    let lit = Litter::new("hello!");

    assert_eq!("hello!", lit.read());
}

#[test]
fn read_literal() {
    let lit = "hello!".inline();

    assert_eq!("hello!", lit.read());
}

#[test]
#[ignore = "TODO"]
fn write_inline() {
    let lit = Litter::new("hello!");

    assert_eq!("hello!", lit.read());
    lit.write("goodbye!");
    assert_eq!("goodbye!", lit.read());
    lit.write("hello!");
    assert_eq!("hello!", lit.read());
}

#[test]
#[ignore = "TODO"]
fn write_literal() {
    let lit = "hello!".inline();

    assert_eq!("hello!", lit.read());
    lit.write("goodbye!");
    assert_eq!("goodbye!", lit.read());
    lit.write("hello!");
    assert_eq!("hello!", lit.read());
}
