use toke;

#[test]
fn test_toke() {
    let doc = toke::Document::parse(
        r#"
        fn main() {
            println!("Hello, world!");
        }
        "#,
    );

    dbg!(&doc);
}
