//! Integration tests for stringify_verbatim
//!
//! These tests verify whitespace preservation behavior and document limitations.

use stringify_verbatim::stringify_verbatim;

#[test]
fn test_basic_spacing() {
    // Multiple spaces between tokens should be preserved
    let s = stringify_verbatim!(foo   bar);
    assert_eq!(s, "foo   bar", "Multiple spaces should be preserved");
}

#[test]
fn test_single_token() {
    let s = stringify_verbatim!(hello);
    assert_eq!(s, "hello");
}

#[test]
fn test_operators() {
    let s = stringify_verbatim!(a + b);
    // Note: spacing around operators
    println!("operators: {:?}", s);
    assert!(s.contains("+"));
}

#[test]
fn test_no_tokens() {
    let s = stringify_verbatim!();
    assert_eq!(s, "");
}

#[test]
fn test_multiline() {
    let s = stringify_verbatim!(
        line1
        line2
    );
    println!("multiline: {:?}", s);
    assert!(s.contains("line1"), "Should contain line1");
    assert!(s.contains("line2"), "Should contain line2");
    // Check if newline is preserved
    assert!(s.contains('\n'), "Should contain newline");
}

#[test]
fn test_complex_rust_code() {
    let s = stringify_verbatim!(
        fn example() {
            42
        }
    );
    println!("complex: {:?}", s);
    assert!(s.contains("fn"));
    assert!(s.contains("example"));
    assert!(s.contains("42"));
}

#[test]
fn test_nested_groups() {
    let s = stringify_verbatim!((a, b, c));
    println!("nested: {:?}", s);
    assert!(s.starts_with("("));
    assert!(s.ends_with(")"));
}

#[test]
fn test_string_literal() {
    let s = stringify_verbatim!("hello world");
    assert_eq!(s, "\"hello world\"");
}

#[test]
fn test_indentation_preservation() {
    // This tests whether indentation on subsequent lines is preserved
    let s = stringify_verbatim!(
        outer
            inner
    );
    println!("indentation: {:?}", s);
    // The inner line should have more leading spaces than outer
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() >= 2 {
        let outer_indent = lines[0].len() - lines[0].trim_start().len();
        let inner_indent = lines[1].len() - lines[1].trim_start().len();
        println!("outer_indent={}, inner_indent={}", outer_indent, inner_indent);
        // Inner should be more indented than outer
        assert!(inner_indent > outer_indent, "Inner should be more indented");
    }
}

/// This test documents the leading whitespace limitation.
/// The first token's position is relative to the source file line, not the macro.
#[test]
fn test_leading_whitespace_limitation() {
    // We can't capture leading whitespace before the first token
    // because spans are relative to the source line, not the macro delimiter
    let s = stringify_verbatim!(   first   second);
    println!("leading ws test: {:?}", s);
    // The leading spaces before "first" will likely NOT be captured
    // This documents the limitation
    assert!(s.contains("first"));
    assert!(s.contains("second"));
    // Check if the spacing between tokens is preserved
    assert!(s.contains("   "), "Spacing between tokens should be preserved");
}

#[test]
fn test_compare_with_stringify() {
    // Show the difference between stringify! and stringify_verbatim!
    let standard = stringify!(foo   bar   baz);
    let verbatim = stringify_verbatim!(foo   bar   baz);

    println!("standard stringify: {:?}", standard);
    println!("verbatim stringify: {:?}", verbatim);

    // Standard stringify normalizes to single spaces
    assert_eq!(standard, "foo bar baz", "Standard stringify normalizes whitespace");

    // Verbatim preserves multiple spaces
    assert_eq!(verbatim, "foo   bar   baz", "Verbatim preserves whitespace");
}

/// Test that regular comments are completely lost (not part of token stream)
#[test]
fn test_regular_comments_lost() {
    let s = stringify_verbatim!(
        foo // this comment is lost
        bar
    );
    println!("with line comment: {:?}", s);
    assert!(s.contains("foo"), "Should contain foo");
    assert!(s.contains("bar"), "Should contain bar");
    assert!(!s.contains("comment"), "Regular comments should NOT be preserved");
    assert!(!s.contains("//"), "Comment syntax should NOT appear");
}

#[test]
fn test_block_comments_lost() {
    let s = stringify_verbatim!(
        foo /* block comment */ bar
    );
    println!("with block comment: {:?}", s);
    assert!(s.contains("foo"), "Should contain foo");
    assert!(s.contains("bar"), "Should contain bar");
    assert!(!s.contains("block"), "Block comments should NOT be preserved");
    assert!(!s.contains("/*"), "Comment syntax should NOT appear");
}

/// Doc comments ARE part of the token stream and are round-tripped back to /// format
#[test]
fn test_doc_comments_round_trip() {
    let s = stringify_verbatim!(
        /// This is a doc comment
        fn example() {}
    );
    println!("with doc comment: {:?}", s);

    // Doc comments are detected and converted back to /// format
    assert!(s.contains("///"), "Doc comment should be round-tripped to ///");
    assert!(s.contains("This is a doc comment"), "Doc content should be preserved");
    assert!(s.contains("fn example"), "Function should be preserved");

    // The /// syntax IS preserved through round-tripping
    assert!(s.contains("/// This is a doc comment"), "Full doc comment should be present");
}

#[test]
fn test_inner_doc_comments() {
    let s = stringify_verbatim!(
        //! Inner doc comment
        mod example {}
    );
    println!("with inner doc: {:?}", s);

    // Inner doc comments are round-tripped back to //! format
    assert!(s.contains("//!"), "Inner doc should be round-tripped to //!");
    assert!(s.contains("//! Inner doc comment"), "Inner doc content should be preserved");
}

/// Test various punctuation and operators
#[test]
fn test_punctuation_spacing() {
    let s = stringify_verbatim!(a + b - c * d / e);
    println!("operators: {:?}", s);
    // Check operators are present
    assert!(s.contains("+"));
    assert!(s.contains("-"));
    assert!(s.contains("*"));
    assert!(s.contains("/"));
}

#[test]
fn test_comma_separated() {
    let s = stringify_verbatim!(a, b, c);
    println!("comma separated: {:?}", s);
    assert!(s.contains(","));
}

#[test]
fn test_path_syntax() {
    let s = stringify_verbatim!(std::collections::HashMap);
    println!("path: {:?}", s);
    assert!(s.contains("std"));
    assert!(s.contains("::"));
    assert!(s.contains("HashMap"));
}

#[test]
fn test_generics() {
    let s = stringify_verbatim!(Vec<String>);
    println!("generics: {:?}", s);
    assert!(s.contains("Vec"));
    assert!(s.contains("<"));
    assert!(s.contains(">"));
    assert!(s.contains("String"));
}

#[test]
fn test_lifetime() {
    let s = stringify_verbatim!(&'a str);
    println!("lifetime: {:?}", s);
    assert!(s.contains("'a"));
    assert!(s.contains("str"));
}

// =============================================================================
// Doc comment round-trip tests
// =============================================================================

/// Test exact output format of doc comments
#[test]
fn test_doc_comment_exact_format() {
    let s = stringify_verbatim!(
        /// Single line doc
        fn foo() {}
    );
    println!("doc comment exact: {:?}", s);
    // Doc comments are round-tripped back to /// format
    assert!(s.starts_with("///"), "Should start with ///");
    assert!(s.contains("/// Single line doc"), "Should preserve doc comment content");
    assert!(s.contains("fn foo()"), "Should contain the function");
}

#[test]
fn test_doc_comment_multiline() {
    let s = stringify_verbatim!(
        /// First line
        /// Second line
        fn foo() {}
    );
    println!("multiline doc: {:?}", s);
    // Each /// line is round-tripped back to /// format
    assert!(s.contains("/// First line"), "First doc line should be preserved");
    assert!(s.contains("/// Second line"), "Second doc line should be preserved");
    // Both doc comments should appear before the function
    let first_pos = s.find("/// First line").unwrap();
    let second_pos = s.find("/// Second line").unwrap();
    let fn_pos = s.find("fn foo").unwrap();
    assert!(first_pos < second_pos, "First line should come before second");
    assert!(second_pos < fn_pos, "Doc comments should come before function");
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
    println!("doc with code block: {:?}", s);
    // Doc comments with code blocks should be preserved
    assert!(s.contains("/// Example:"), "Doc comment intro should be preserved");
    assert!(s.contains("/// ```"), "Code fence should be preserved");
    assert!(s.contains("/// let x = 1;"), "Code content should be preserved");
}

#[test]
fn test_inner_doc_exact_format() {
    let s = stringify_verbatim!(
        //! Module doc
        mod foo {}
    );
    println!("inner doc exact: {:?}", s);
    // Inner doc comments are round-tripped back to //! format
    assert!(s.starts_with("//!"), "Should start with //!");
    assert!(s.contains("//! Module doc"), "Should preserve inner doc content");
    assert!(s.contains("mod foo"), "Should contain the module");
}

/// Test that we can detect doc attributes and their content
#[test]
fn test_doc_attribute_detection() {
    let s = stringify_verbatim!(
        /// This is documentation
        fn example() {}
    );

    // The output should contain the doc content
    assert!(s.contains("This is documentation"), "Doc content should be preserved");

    // Check current format (will update based on implementation)
    println!("Doc attribute output: {:?}", s);
}

/// Test consistency: same input should always produce same output
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
    assert_eq!(s1, s2, "Same input should produce same output");
}

/// Test that explicit #[doc] attributes are NOT converted to /// format
/// Only actual /// comments are round-tripped, explicit attributes stay as-is
#[test]
fn test_explicit_doc_attribute() {
    let s = stringify_verbatim!(
        #[doc = "Explicit doc"]
        fn foo() {}
    );
    println!("explicit doc attr: {:?}", s);
    // Explicit #[doc = "..."] should NOT be converted to /// format
    // It should remain as #[doc = "..."] because it was written that way
    assert!(s.contains("#"), "Explicit doc should keep # prefix");
    assert!(s.contains("doc"), "Should contain doc");
    assert!(s.contains("Explicit doc"), "Doc content should be preserved");
    // Should NOT have been converted to ///
    assert!(!s.starts_with("///"), "Explicit attr should NOT become ///");
}

/// Document the difference between /// and #[doc = "..."]
/// Only /// comments are round-tripped, explicit attributes remain as-is
#[test]
fn test_doc_comment_vs_explicit_attribute_behavior() {
    // When you write `/// Text`, it's round-tripped back to ///
    let from_triple_slash = stringify_verbatim!(
        /// Text with space
        fn a() {}
    );

    // When you write `#[doc = "Text"]` explicitly, it stays as #[doc = "..."]
    let from_explicit = stringify_verbatim!(
        #[doc = "Text without space"]
        fn b() {}
    );

    println!("from ///: {:?}", from_triple_slash);
    println!("from #[doc]: {:?}", from_explicit);

    // /// comments are converted back to /// format
    assert!(from_triple_slash.contains("/// Text with space"), "/// should round-trip to ///");

    // Explicit #[doc = "..."] should NOT be converted
    assert!(from_explicit.contains("#"), "Explicit should keep #");
    assert!(from_explicit.contains("doc"), "Explicit should keep doc");
    assert!(!from_explicit.starts_with("///"), "Explicit should NOT become ///");
}

/// Test that non-doc attributes are NOT converted (only doc attributes are round-tripped)
#[test]
fn test_non_doc_attributes_not_converted() {
    let s = stringify_verbatim!(
        #[derive(Debug)]
        #[allow(unused)]
        struct Foo {}
    );
    println!("non-doc attrs: {:?}", s);
    // Non-doc attributes should remain in attribute format
    assert!(s.contains("#"), "Attribute should keep # prefix");
    assert!(s.contains("derive"), "Should contain derive");
    assert!(s.contains("Debug"), "Should contain Debug");
    assert!(s.contains("allow"), "Should contain allow");
    assert!(s.contains("unused"), "Should contain unused");
}

/// Test mixing doc and non-doc attributes
#[test]
fn test_mixed_doc_and_other_attributes() {
    let s = stringify_verbatim!(
        /// Documentation
        #[derive(Clone)]
        struct Bar {}
    );
    println!("mixed attrs: {:?}", s);
    // Doc comment should be round-tripped to ///
    assert!(s.contains("/// Documentation"), "Doc comment should be ///");
    // But #[derive(Clone)] should remain as attribute
    assert!(s.contains("derive"), "Derive should remain");
    assert!(s.contains("Clone"), "Clone should remain");
}
