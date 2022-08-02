use toke;

pub fn main() {
    let literal = toke::n!(def fn hello world (you say "goodbye"))
        .walk()
        .find(|x| x.node_type() == toke::TokenType::Literal)
        .unwrap();
    assert_eq!("\"goodbye\"", literal.as_str());
}
