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
