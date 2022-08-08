use token_tree::{self, document, Node};

#[test]
fn test_toke() -> Result<(), miette::Report> {
    let doc = document! {
        static message: &str = "Hello, world!";

        pub fn main() {
            println!("{message}");
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
