use toke::{self, Node};

#[test]
fn test_toke() -> Result<(), miette::Report> {
    let doc = toke::Document::parse(
        r#"
        fn main() {
            println!("Hello, world!");
        }
        "#,
    )
    .unwrap();

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
