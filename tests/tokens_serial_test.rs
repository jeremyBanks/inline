//! Integration tests for tokens!() macro
//!
//! These tests verify that the tokens! macro works correctly with the inline cell system,
//! including updating token contents and persisting them to source files.

use inline::{InlineCellPrivate, Tokens};
use std::env;
use std::fs;
use tempfile::TempDir;

/// Helper to find tokens! macro invocations in a file
fn find_tokens_macro_positions(path: &std::path::Path) -> Vec<(u32, u32)> {
    let source = fs::read_to_string(path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct MacroFinder {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for MacroFinder {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if let syn::Expr::Macro(mac) = node {
                if let Some(seg) = mac.mac.path.segments.last() {
                    let name = seg.ident.to_string();
                    if name == "tokens" {
                        use syn::spanned::Spanned;
                        let span = mac.mac.path.span();
                        let start = span.start();
                        self.positions.push((start.line as u32, start.column as u32));
                    }
                }
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut finder = MacroFinder { positions: Vec::new() };
    finder.visit_file(&ast);
    finder.positions
}

#[test]
fn test_tokens_macro_basic() {
    // Test that tokens!() creates an InlineCell<Tokens> and updates correctly
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = tokens!(foo bar 123);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_tokens_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one tokens! macro");
    let (line, col) = positions[0];

    {
        let initial = Tokens::from_str("foo bar 123");
        let mut cell = inline::InlineCell::__new(initial, path.to_str().unwrap(), line, col);
        cell.value = Tokens::from_str("new_value 456");
        // Drop triggers write
    }

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("new_value 456"),
        "File should contain updated tokens. Actual:\n{}",
        content
    );
    // The macro wrapper should be preserved
    assert!(
        content.contains("tokens!("),
        "Macro syntax should be preserved. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_tokens_macro_complex_tokens() {
    // Test with more complex Rust-like tokens
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = tokens!(fn example() -> i32 { 42 });
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_tokens_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one tokens! macro");
    let (line, col) = positions[0];

    {
        let initial = Tokens::from_str("fn example() -> i32 { 42 }");
        let mut cell = inline::InlineCell::__new(initial, path.to_str().unwrap(), line, col);
        cell.value = Tokens::from_str("fn updated() -> String { \"hello\".into() }");
    }

    let content = fs::read_to_string(&path).unwrap();
    // Token normalization adds spaces, so check for the key parts
    assert!(
        content.contains("fn updated"),
        "File should contain updated function. Actual:\n{}",
        content
    );
    assert!(
        content.contains("String"),
        "File should contain return type. Actual:\n{}",
        content
    );
    assert!(
        content.contains("tokens!("),
        "Macro syntax should be preserved. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_tokens_macro_with_braces() {
    // Test that brace delimiters are preserved
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = tokens!{foo bar};
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_tokens_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one tokens! macro");
    let (line, col) = positions[0];

    {
        let initial = Tokens::from_str("foo bar");
        let mut cell = inline::InlineCell::__new(initial, path.to_str().unwrap(), line, col);
        cell.value = Tokens::from_str("baz qux");
    }

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("baz qux"),
        "File should contain updated tokens. Actual:\n{}",
        content
    );
    assert!(
        content.contains("tokens!{"),
        "Brace style should be preserved. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_tokens_macro_with_brackets() {
    // Test that bracket delimiters are preserved
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = tokens![a b c];
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_tokens_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one tokens! macro");
    let (line, col) = positions[0];

    {
        let initial = Tokens::from_str("a b c");
        let mut cell = inline::InlineCell::__new(initial, path.to_str().unwrap(), line, col);
        cell.value = Tokens::from_str("x y z");
    }

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("x y z"),
        "File should contain updated tokens. Actual:\n{}",
        content
    );
    assert!(
        content.contains("tokens!["),
        "Bracket style should be preserved. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_tokens_macro_memory_mode() {
    // In memory mode, tokens! should work without file writes
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = tokens!(original value);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "memory");
    inline::clear_file_state_cache();

    let positions = find_tokens_macro_positions(&path);
    let (line, col) = positions[0];

    {
        let initial = Tokens::from_str("original value");
        let mut cell = inline::InlineCell::__new(initial, path.to_str().unwrap(), line, col);
        cell.value = Tokens::from_str("modified value");
    }

    // File should NOT be modified in memory mode
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("tokens!(original value)"),
        "File should still contain original. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_tokens_macro_empty() {
    // Test with empty tokens
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = tokens!();
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_tokens_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one tokens! macro");
    let (line, col) = positions[0];

    {
        let initial = Tokens::from_str("");
        let mut cell = inline::InlineCell::__new(initial, path.to_str().unwrap(), line, col);
        cell.value = Tokens::from_str("now has content");
    }

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("now has content"),
        "File should contain new tokens. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_tokens_macro_with_quote() {
    // Test that tokens created via quote! work correctly
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = tokens!(initial);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_tokens_macro_positions(&path);
    let (line, col) = positions[0];

    {
        let initial = Tokens::from_str("initial");
        let mut cell = inline::InlineCell::__new(initial, path.to_str().unwrap(), line, col);
        // Use quote! to generate new tokens
        use quote::quote;
        cell.value = quote!(struct Generated { field: i32 }).into();
    }

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("struct Generated"),
        "File should contain quote!-generated tokens. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_tokens_macro_verify_mode() {
    // In verify mode, mismatched tokens should cause an error
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = tokens!(expected value);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "verify");
    inline::clear_file_state_cache();

    let positions = find_tokens_macro_positions(&path);
    let (line, col) = positions[0];

    // Create a cell with matching value - should succeed
    {
        let initial = Tokens::from_str("expected value");
        let cell = inline::InlineCell::__new(initial.clone(), path.to_str().unwrap(), line, col);
        // Value matches, drop should succeed
        assert_eq!(cell.value, initial);
    }

    env::remove_var("INLINE_MODE");
}
