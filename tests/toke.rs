use toke;

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

    let root = doc.root();

    let items = root.children();

    let body = &items[3];

    dbg!(body);

    panic!()
}
