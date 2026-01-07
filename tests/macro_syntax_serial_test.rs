//! Tests for macro syntax: cell!() and replace!()
//!
//! These tests verify that the macro versions work identically to the function versions.

use inline::InlineCellPrivate;
use std::env;
use std::fs;
use tempfile::TempDir;

/// Helper to find macro invocations positions in a file
fn find_macro_positions(path: &std::path::Path) -> Vec<(u32, u32)> {
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
                    if name == "cell" || name == "replace" {
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
fn test_cell_macro_basic() {
    // Test that cell!() works like cell()
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = cell!(42u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one macro");
    let (line, col) = positions[0];

    {
        let mut cell = inline::InlineCell::__new(42u32, path.to_str().unwrap(), line, col);
        cell.value = 100u32;
        // Drop triggers write
    }

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("100u32"),
        "File should contain updated value. Actual:\n{}",
        content
    );
    // The macro wrapper should be preserved
    assert!(
        content.contains("cell!("),
        "Macro syntax should be preserved. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_cell_macro_with_expression() {
    // Test cell!() with a more complex expression
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = cell!(1 + 2);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one macro");
    let (line, col) = positions[0];

    {
        // The expression `1 + 2` evaluates to 3
        let mut cell = inline::InlineCell::__new(3i32, path.to_str().unwrap(), line, col);
        cell.value = 10i32;
    }

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("10i32"),
        "File should contain updated value. Actual:\n{}",
        content
    );
    // The macro wrapper should still be there
    assert!(
        content.contains("cell!("),
        "Macro syntax should be preserved. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_cell_macro_with_braces() {
    // Test that different delimiter styles work
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = cell!{5u32};
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one macro");
    let (line, col) = positions[0];

    {
        let mut cell = inline::InlineCell::__new(5u32, path.to_str().unwrap(), line, col);
        cell.value = 99u32;
    }

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("99u32"),
        "File should contain updated value. Actual:\n{}",
        content
    );
    // The braces should be preserved
    assert!(
        content.contains("cell!{"),
        "Brace style should be preserved. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_replace_macro_basic() {
    // Test that replace!() replaces the entire macro invocation
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = replace!(42u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "write");
    inline::clear_file_state_cache();

    let positions = find_macro_positions(&path);
    assert_eq!(positions.len(), 1, "Should find one macro");
    let (line, col) = positions[0];

    // Use replace_at to trigger the replacement
    let result = inline::replace_at(100u32, path.to_str().unwrap(), line, col);
    assert_eq!(result, 100u32);

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("100u32"),
        "File should contain the baked value. Actual:\n{}",
        content
    );
    assert!(
        !content.contains("replace!"),
        "Macro invocation should be replaced. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_cell_macro_memory_mode() {
    // In memory mode, macros should work without file writes
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = cell!(42u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("INLINE_MODE", "memory");
    inline::clear_file_state_cache();

    let positions = find_macro_positions(&path);
    let (line, col) = positions[0];

    {
        let mut cell = inline::InlineCell::__new(42u32, path.to_str().unwrap(), line, col);
        cell.value = 100u32;
    }

    // File should NOT be modified in memory mode
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("cell!(42u32)"),
        "File should still contain original. Actual:\n{}",
        content
    );

    env::remove_var("INLINE_MODE");
}
