//! Tests for things that ARE preserved by stringify_verbatim (lossless round-trip)
//!
//! These tests document what stringify_verbatim successfully preserves from the original source.

use stringify_verbatim::stringify_verbatim;

// =============================================================================
// Whitespace preservation
// =============================================================================

#[test]
fn test_multiple_spaces_between_tokens() {
    let s = stringify_verbatim!(foo   bar);
    assert_eq!(s, "foo   bar", "Multiple spaces should be preserved");
}

#[test]
fn test_newlines_preserved() {
    let s = stringify_verbatim!(
        line1
        line2
    );
    assert!(s.contains("line1"));
    assert!(s.contains("line2"));
    assert!(s.contains('\n'), "Newlines should be preserved");
}

#[test]
fn test_indentation_preserved() {
    let s = stringify_verbatim!(
        outer
            inner
    );
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() >= 2 {
        let outer_indent = lines[0].len() - lines[0].trim_start().len();
        let inner_indent = lines[1].len() - lines[1].trim_start().len();
        assert!(inner_indent > outer_indent, "Relative indentation should be preserved");
    }
}

#[test]
fn test_complex_whitespace_in_code() {
    let s = stringify_verbatim!(
        fn example() {
            42
        }
    );
    assert!(s.contains("fn"));
    assert!(s.contains("example"));
    assert!(s.contains("42"));
    assert!(s.contains('\n'), "Newlines in code should be preserved");
}

// =============================================================================
// Token preservation
// =============================================================================

#[test]
fn test_identifiers() {
    let s = stringify_verbatim!(hello);
    assert_eq!(s, "hello");
}

#[test]
fn test_operators() {
    let s = stringify_verbatim!(a + b - c * d / e);
    assert!(s.contains("+"));
    assert!(s.contains("-"));
    assert!(s.contains("*"));
    assert!(s.contains("/"));
}

#[test]
fn test_punctuation() {
    let s = stringify_verbatim!(a, b, c);
    assert!(s.contains(","));
}

#[test]
fn test_path_syntax() {
    let s = stringify_verbatim!(std::collections::HashMap);
    assert!(s.contains("std"));
    assert!(s.contains("::"));
    assert!(s.contains("HashMap"));
}

#[test]
fn test_generics() {
    let s = stringify_verbatim!(Vec<String>);
    assert!(s.contains("Vec"));
    assert!(s.contains("<"));
    assert!(s.contains(">"));
    assert!(s.contains("String"));
}

#[test]
fn test_lifetimes() {
    let s = stringify_verbatim!(&'a str);
    assert!(s.contains("'a"));
    assert!(s.contains("str"));
}

#[test]
fn test_string_literals() {
    let s = stringify_verbatim!("hello world");
    assert_eq!(s, "\"hello world\"");
}

#[test]
fn test_nested_groups() {
    let s = stringify_verbatim!((a, b, c));
    assert!(s.starts_with("("));
    assert!(s.ends_with(")"));
}

// =============================================================================
// Doc comment round-tripping (/// and //!)
// =============================================================================

#[test]
fn test_doc_comment_triple_slash() {
    let s = stringify_verbatim!(
        /// This is a doc comment
        fn example() {}
    );
    assert!(s.contains("/// This is a doc comment"), "/// comments should round-trip");
    assert!(s.contains("fn example"), "Code after doc should be preserved");
}

#[test]
fn test_doc_comment_multiline() {
    let s = stringify_verbatim!(
        /// First line
        /// Second line
        fn foo() {}
    );
    assert!(s.contains("/// First line"), "First doc line preserved");
    assert!(s.contains("/// Second line"), "Second doc line preserved");

    // Order should be preserved
    let first_pos = s.find("/// First line").unwrap();
    let second_pos = s.find("/// Second line").unwrap();
    let fn_pos = s.find("fn foo").unwrap();
    assert!(first_pos < second_pos);
    assert!(second_pos < fn_pos);
}

#[test]
fn test_doc_comment_with_code_block() {
    let s = stringify_verbatim!(
        /// Example:
        /// ```
        /// let x = 1;
        /// ```
        fn foo() {}
    );
    assert!(s.contains("/// Example:"));
    assert!(s.contains("/// ```"));
    assert!(s.contains("/// let x = 1;"));
}

#[test]
fn test_inner_doc_comment() {
    let s = stringify_verbatim!(
        //! Inner doc comment
        mod example {}
    );
    assert!(s.contains("//! Inner doc comment"), "//! comments should round-trip");
}

#[test]
fn test_doc_comment_consistency() {
    let s1 = stringify_verbatim!(
        /// Doc comment
        fn foo() {}
    );
    let s2 = stringify_verbatim!(
        /// Doc comment
        fn foo() {}
    );
    assert_eq!(s1, s2, "Same input should produce identical output");
}

#[test]
fn test_doc_comment_with_escape_sequences() {
    // Doc comments can contain escape sequences
    let s = stringify_verbatim!(
        /// This has a "quote" in it
        fn foo() {}
    );
    println!("doc with quotes: {:?}", s);
    // The quote should appear as a literal quote, not as \"
    assert!(s.contains(r#"/// This has a "quote" in it"#),
        "Escape sequences in doc comments should be unescaped");
}

#[test]
fn test_doc_comment_with_backslash() {
    // Doc comments can contain backslashes
    let s = stringify_verbatim!(
        /// Path: C:\Users\test
        fn foo() {}
    );
    println!("doc with backslash: {:?}", s);
    // Backslashes should appear correctly
    assert!(s.contains(r"/// Path: C:\Users\test"),
        "Backslashes in doc comments should be preserved");
}

#[test]
fn test_doc_comment_backslash_n_is_literal() {
    // In doc comments, \n is literal backslash-n, NOT a newline escape
    // (Doc comments are not string literals - escape sequences are not processed)
    let s = stringify_verbatim!(
        /// Line1\nLine2
        fn foo() {}
    );
    println!("doc with backslash-n: {:?}", s);
    // Should contain literal backslash-n, not a newline
    assert!(s.contains(r"/// Line1\nLine2"),
        "Backslash-n in doc comments is literal, not an escape");
}

#[test]
fn test_doc_comment_backslash_t_is_literal() {
    // In doc comments, \t is literal backslash-t, NOT a tab escape
    let s = stringify_verbatim!(
        /// Col1\tCol2
        fn foo() {}
    );
    println!("doc with backslash-t: {:?}", s);
    // Should contain literal backslash-t, not a tab
    assert!(s.contains(r"/// Col1\tCol2"),
        "Backslash-t in doc comments is literal, not an escape");
}

#[test]
fn test_doc_comment_backslash_u_is_literal() {
    // In doc comments, \u{...} is literal text, NOT a unicode escape
    let s = stringify_verbatim!(
        /// Heart: \u{2764}
        fn foo() {}
    );
    println!("doc with backslash-u: {:?}", s);
    // Should contain literal \u{2764}, not the heart character
    assert!(s.contains(r"/// Heart: \u{2764}"),
        "Backslash-u in doc comments is literal, not an escape");
}

// =============================================================================
// Explicit attributes (preserved as-is, not converted)
// =============================================================================

#[test]
fn test_explicit_doc_attribute_preserved() {
    let s = stringify_verbatim!(
        #[doc = "Explicit doc"]
        fn foo() {}
    );
    // Explicit #[doc = "..."] should remain as #[doc = "..."], NOT become ///
    assert!(s.contains("#"), "Explicit attr keeps #");
    assert!(s.contains("doc"), "Explicit attr keeps doc");
    assert!(s.contains("Explicit doc"), "Content preserved");
    assert!(!s.starts_with("///"), "Should NOT become ///");
}

#[test]
fn test_derive_attributes_preserved() {
    let s = stringify_verbatim!(
        #[derive(Debug)]
        #[allow(unused)]
        struct Foo {}
    );
    assert!(s.contains("#"));
    assert!(s.contains("derive"));
    assert!(s.contains("Debug"));
    assert!(s.contains("allow"));
    assert!(s.contains("unused"));
}

#[test]
fn test_mixed_doc_and_attributes() {
    let s = stringify_verbatim!(
        /// Documentation
        #[derive(Clone)]
        struct Bar {}
    );
    // Doc comment should be round-tripped to ///
    assert!(s.contains("/// Documentation"), "Doc comment becomes ///");
    // But #[derive(Clone)] should remain as attribute
    assert!(s.contains("derive"));
    assert!(s.contains("Clone"));
}

#[test]
fn test_doc_comment_vs_explicit_attribute() {
    // /// syntax round-trips to ///
    let from_triple_slash = stringify_verbatim!(
        /// Text with space
        fn a() {}
    );

    // #[doc = "..."] stays as #[doc = "..."]
    let from_explicit = stringify_verbatim!(
        #[doc = "Text without space"]
        fn b() {}
    );

    assert!(from_triple_slash.contains("/// Text with space"), "/// round-trips");
    assert!(from_explicit.contains("#"), "Explicit stays as attr");
    assert!(!from_explicit.starts_with("///"), "Explicit doesn't become ///");
}
