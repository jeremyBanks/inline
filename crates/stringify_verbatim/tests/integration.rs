//! Basic integration tests for stringify_verbatim
//!
//! For detailed tests of what IS and IS NOT preserved, see:
//! - preserved.rs: Things that round-trip losslessly
//! - not_preserved.rs: Things that are lost during tokenization

use stringify_verbatim::stringify_verbatim;

#[test]
fn test_empty_input() {
    let s = stringify_verbatim!();
    assert_eq!(s, "");
}

#[test]
fn test_single_token() {
    let s = stringify_verbatim!(hello);
    assert_eq!(s, "hello");
}

#[test]
fn test_basic_spacing() {
    let s = stringify_verbatim!(foo   bar);
    assert_eq!(s, "foo   bar");
}

/// Compare with standard stringify! to show the difference
#[test]
fn test_compare_with_stringify() {
    let standard = stringify!(foo   bar   baz);
    let verbatim = stringify_verbatim!(foo   bar   baz);

    // Standard stringify normalizes to single spaces
    assert_eq!(standard, "foo bar baz", "Standard stringify normalizes whitespace");

    // Verbatim preserves multiple spaces
    assert_eq!(verbatim, "foo   bar   baz", "Verbatim preserves whitespace");
}

/// Verify multiline code works
#[test]
fn test_multiline_code() {
    let s = stringify_verbatim!(
        fn example() {
            42
        }
    );
    assert!(s.contains("fn example"));
    assert!(s.contains("42"));
    assert!(s.contains('\n'));
}
