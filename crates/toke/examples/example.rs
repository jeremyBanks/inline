use toke;

pub fn main() {
    let literal = toke::n!(def fn hello world (you say "goodbye"))
        .iter()
        .find(|x| x.node_type() == toke::NodeType::Literal)
        .unwrap();
    assert_eq!("\"goodbye\"", literal.as_str());
}
