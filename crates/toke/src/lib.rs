//! 🌳 `toke` provides a read-only graph interface for traversing Rust syntax files.
#![doc(html_favicon_url = "https://static.jeremy.ca/toke.png")]
#![doc(html_logo_url = "https://static.jeremy.ca/toke.png")]
#![cfg_attr(debug_assertions, allow(unused))]

pub(crate) mod inner;

#[doc(hidden)]
mod outer;
#[doc(inline)]
pub use self::outer::*;

fn _lol() {
    n! {
        // lol
        let document = todo!();
        document.query_selector_all(
            r#"
                #fn + ident + group:parentheses + group:braces > ident#return

            "#,
        )

        r##"
        mod:root
        group() {} []
        ident#hello
        punct. #, #>=
        literal ="hello" =2.0 =i32 =integer =float

        descendants (default)
        children >
        next sibling +
        :first-child
        :last-child
        :only-child
        :empty
        "##
    };
}
