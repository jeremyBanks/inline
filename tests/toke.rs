use toke::{self, attributes, inner_attributes, rs, Node};

#[test]
fn test_toke() -> Result<(), miette::Report> {
    let _attrs = inner_attributes! {
        #![doc = "hello, world!"]
    };

    let doc = rs! {
        fn main() {
            println!("Hello, world!");
        }
    };

    let main = doc.root().first_child().unwrap().next().unwrap();

    let replaced = doc.replace_nodes([(&main, &Node::parse("mane").unwrap())]);

    assert_eq!(
        r#"
        fn mane() {
            println!("Hello, world!");
        }
        "#,
        replaced.as_str()
    );

    Ok(())
}

// should we provide a syn-style .fold on top of replace_nodes and walk?
