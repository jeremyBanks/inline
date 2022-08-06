use {
    std::iter::Filter,
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

    let root = doc.walk().find(|n| n.as_str() == "fn").unwrap();
    // Is NodeWalker where we put our nicer logic?
    // walker.next_group()
    // walker.previous_group()

    // We need several iterators, and various filters on Node iterators.
    // An extension trait is probably a good way to go.

    // but not yet because we're trying to make `litter` work first!
}

// .ancestors()     <-- backwards iterator through parents up to the root
// .following()     <-- forward iterator through all following nodes in the document
// .descendants()   <-- forward iterator through all descendants nodes of this one
// .preceding()     <-- backwards iterator through all preceeding nodes in the document
// .children()      <-- double-ended iterator(?) over all children
// .following_siblings() <-- forward iterator over next siblings
// .preceding_siblings() <-- backwards iterator over previous siblings

// what about following siblings *and* their descendants, or rather, document-order
// walking limited to their parent instead of their own subtree?

trait NodeIteratorExt: Iterator<Item = Node> + Sized {
    fn query(&mut self, selector: impl NodeSelector) -> Option<Node> {
        loop {
            let next = self.next()?;
            if selector.matches(&next) {
                return Some(next);
            }
        }
    }

    fn query_all(self, selector: impl NodeSelector) -> Filter<Self, Box<dyn Fn(&Node) -> bool>> {
        self.filter(Box::new(move |n| selector.matches(n)))
    }
}

trait NodeSelector: 'static {
    fn matches(&self, node: &Node) -> bool;
}

impl NodeSelector for TokenType {
    fn matches(&self, node: &Node) -> bool {
        node.node_type() == *self
    }
}

// TODO: todo!()s and the key feature: replacements! test that!
// use expect_test to validate some current behaviour?

// .closest()

// https://api.jquery.com/category/selectors/
