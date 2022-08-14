use {
    quote::quote,
    token_tree::{self, Document, Node},
};

// #[test]
// fn test_toke() -> Result<(), miette::Report> {
//     let doc: proc_macro2::TokenStream = quote! {
//         static message: &str = "Hello, world!";

//         pub fn main() {
//             println!("{message}");
//         }
//     };

//     let doc: Document = doc.into();

//     let main = doc.root().first_child().unwrap().next().unwrap();

//     let replaced = doc.replace_nodes([(&main, &Node::parse("mane").unwrap())]);

//     assert_eq!(
//         r#"
//         fn mane() {
//             println!("Hello, world!");
//         }
//         "#,
//         replaced.as_str()
//     );

//     Ok(())
// }
