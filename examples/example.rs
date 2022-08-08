use token_tree;

pub fn main() {
    let literal = token_tree::node!(def fn hello world (you say "goodbye"))
        .walk()
        .find(|x| x.node_type() == token_tree::TokenType::Literal)
        .unwrap();
    assert_eq!("\"goodbye\"", literal.as_str());
}
