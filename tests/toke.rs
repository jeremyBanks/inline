use {
    std::{fmt::Debug, iter::Filter, sync::Arc},
    toke::{self, Node, TokenType},
};

#[test]
fn test_toke() {
    let doc = toke::Document::parse(
        r#"
        fn main() {
            println!("Hello, world!");
        }
        "#,
    )
    .unwrap();
}

#[derive(Clone)]
pub enum Select {
    /// if `node` == `target`
    This,
    Ancestor,
    Sibling,
    Descendant,
    Child,
    /// if all of the selectors match
    All(Vec<Select>),
    /// if any of the selectors match
    Any(Vec<Select>),
    /// if exactly one of the selectors match
    One(Vec<Select>),
    /// if none of the selectors match
    Not(Vec<Select>),
    /// if it matches given the source string exactly
    Source(String),
    Dyn(Arc<dyn NodeSelector>),
    Type(TokenType),
}

impl<'a> NodeSelector for Select {
    fn matches(&self, target: &Node, origin: &Node) -> bool {
        use Select::*;
        match self {
            This => target == origin,
            All(s) => s.iter().all(|select| select.matches(target, origin)),
            Any(s) => s.iter().any(|select| select.matches(target, origin)),
            Dyn(s) => s.matches(target, origin),
            One(s) =>
                s.iter()
                    .filter(|select| select.matches(target, origin))
                    .take(2)
                    .count()
                    == 1,
            Not(s) => !s.iter().any(|select| select.matches(target, origin)),
            Type(t) => target.node_type() == *t,

            #[allow(unreachable_patterns)]
            _ => todo!(),
        }
    }
}

pub trait NodeSelector {
    /// Whether the `target` node matches this selector, for a query starting from `origin`.
    /// Most selectors ignore the `origin`.
    fn matches(&self, target: &Node, origin: &Node) -> bool;
}

impl NodeSelector for fn(&Node) -> bool {
    fn matches(&self, target: &Node, _origin: &Node) -> bool {
        self(target)
    }
}

impl NodeSelector for fn(&Node, &Node) -> bool {
    fn matches(&self, target: &Node, origin: &Node) -> bool {
        self(target, origin)
    }
}

impl NodeSelector for TokenType {
    fn matches(&self, target: &Node, _origin: &Node) -> bool {
        target.node_type() == *self
    }
}

// TODO: todo!()s and the key feature: replacements! test that!
// use expect_test to validate some current behaviour?

// .closest()

// https://api.jquery.com/category/selectors/
