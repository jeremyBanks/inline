//! Tests for things that are NOT preserved by stringify_verbatim (lossy)
//!
//! These tests document the limitations of stringify_verbatim - things that
//! are lost during the tokenization process and cannot be recovered.

use stringify_verbatim::stringify_verbatim;

// =============================================================================
// Regular comments are NOT part of the token stream
// =============================================================================

#[test]
fn test_line_comments_lost() {
    let s = stringify_verbatim!(
        foo // this comment is lost
        bar
    );
    assert!(s.contains("foo"), "Code before comment preserved");
    assert!(s.contains("bar"), "Code after comment preserved");
    assert!(!s.contains("comment"), "Comment text is NOT preserved");
    assert!(!s.contains("//"), "Comment syntax is NOT preserved");
}

#[test]
fn test_block_comments_lost() {
    let s = stringify_verbatim!(
        foo /* block comment */ bar
    );
    assert!(s.contains("foo"), "Code before comment preserved");
    assert!(s.contains("bar"), "Code after comment preserved");
    assert!(!s.contains("block"), "Comment text is NOT preserved");
    assert!(!s.contains("/*"), "Comment syntax is NOT preserved");
}

// =============================================================================
// Leading whitespace before first token
// =============================================================================

#[test]
fn test_leading_whitespace_before_first_token() {
    // Leading whitespace before the first token cannot be captured
    // because span positions are relative to the source file, not the macro
    let s = stringify_verbatim!(   first   second);

    assert!(s.contains("first"));
    assert!(s.contains("second"));
    // The spacing BETWEEN tokens is preserved
    assert!(s.contains("   "), "Spacing between tokens preserved");
    // But leading whitespace before "first" is likely lost
    // (This is a fundamental limitation of span-based reconstruction)
}

// =============================================================================
// Trailing whitespace after last token
// =============================================================================

#[test]
fn test_trailing_whitespace_after_last_token() {
    // Trailing whitespace after the last token cannot be captured
    // because there's no subsequent token to measure distance to
    let s = stringify_verbatim!(token   );

    assert!(s.contains("token"));
    // Trailing spaces after "token" are lost - we can't know they existed
    assert!(!s.ends_with("   "), "Trailing whitespace is NOT preserved");
}

// =============================================================================
// Exact original formatting in some edge cases
// =============================================================================

#[test]
fn test_whitespace_inside_string_literals_is_literal_content() {
    // Note: whitespace INSIDE string literals is the literal's content, not formatting
    // This is preserved because it's part of the token value
    let s = stringify_verbatim!("hello   world");
    assert_eq!(s, "\"hello   world\"", "String content is preserved as token value");
}
