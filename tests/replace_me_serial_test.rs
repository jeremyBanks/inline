//! Tests for replace_me() functionality.
//!
//! These tests verify the one-shot code generation feature.

use code_cell::replace_me_at;
use std::env;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test file with Rust source code
struct TestFile {
    _dir: TempDir,
    pub path: std::path::PathBuf,
}

impl TestFile {
    fn new(content: &str) -> Self {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.rs");
        fs::write(&path, content).unwrap();
        TestFile { _dir: dir, path }
    }

    fn read(&self) -> String {
        fs::read_to_string(&self.path).unwrap()
    }

    fn assert_contains(&self, s: &str) {
        assert!(
            self.read().contains(s),
            "Expected file to contain: {}\n\nActual content:\n{}",
            s,
            self.read()
        );
    }

    fn assert_does_not_contain(&self, s: &str) {
        assert!(
            !self.read().contains(s),
            "Expected file to NOT contain: {}\n\nActual content:\n{}",
            s,
            self.read()
        );
    }
}

/// Helper to find the position of a function call in a file
fn find_call_position(file_path: &std::path::Path) -> (u32, u32) {
    let source = fs::read_to_string(file_path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct CallFinder {
        position: Option<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for CallFinder {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if self.position.is_none() {
                if let syn::Expr::Call(call) = node {
                    use syn::spanned::Spanned;
                    let span = call.func.span();
                    let start = span.start();
                    self.position = Some((start.line as u32, start.column as u32));
                }
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut finder = CallFinder { position: None };
    finder.visit_file(&ast);
    finder.position.expect("Should find a function call")
}

#[test]
fn test_replace_me_basic() {
    // Test that replace_me replaces the entire expression
    let test_file = TestFile::new(
        r#"fn main() {
    let x = replace_me(42u32);
}
"#,
    );

    env::set_var("CODE_CELL_MODE", "write");

    // Clear any cached state
    code_cell::clear_file_state_cache();

    let (line, column) = find_call_position(&test_file.path);

    // Call replace_me_at - this should replace the entire expression
    let result = replace_me_at(100u32, test_file.path.to_str().unwrap(), line, column);

    // Should return the value
    assert_eq!(result, 100u32);

    // The file should now contain the baked value, not the replace_me call
    test_file.assert_contains("100u32");
    test_file.assert_does_not_contain("replace_me");

    env::remove_var("CODE_CELL_MODE");
}

#[test]
fn test_replace_me_memory_mode() {
    // In memory mode, no file writes should happen
    let test_file = TestFile::new(
        r#"fn main() {
    let x = replace_me(42u32);
}
"#,
    );

    env::set_var("CODE_CELL_MODE", "memory");

    // Clear any cached state
    code_cell::clear_file_state_cache();

    let (line, column) = find_call_position(&test_file.path);

    // Call replace_me_at
    let result = replace_me_at(100u32, test_file.path.to_str().unwrap(), line, column);

    // Should return the value
    assert_eq!(result, 100u32);

    // File should still contain the original replace_me call
    test_file.assert_contains("replace_me(42u32)");

    env::remove_var("CODE_CELL_MODE");
}

#[test]
fn test_replace_me_persistence() {
    // Multiple calls from the same location should return the same value
    let test_file = TestFile::new(
        r#"fn main() {
    let x = replace_me(42u32);
}
"#,
    );

    env::set_var("CODE_CELL_MODE", "memory");

    // Clear any cached state
    code_cell::clear_file_state_cache();

    let (line, column) = find_call_position(&test_file.path);

    // First call stores the value
    let result1 = replace_me_at(100u32, test_file.path.to_str().unwrap(), line, column);
    assert_eq!(result1, 100u32);

    // Second call from same location returns the stored value, ignoring the new argument
    let result2 = replace_me_at(999u32, test_file.path.to_str().unwrap(), line, column);
    assert_eq!(result2, 100u32); // Should still be 100, not 999

    env::remove_var("CODE_CELL_MODE");
}

#[test]
fn test_replace_me_different_locations() {
    // Different locations should get different values
    let test_file = TestFile::new(
        r#"fn main() {
    let x = replace_me(10u32);
    let y = replace_me(20u32);
}
"#,
    );

    env::set_var("CODE_CELL_MODE", "memory");

    // Clear any cached state
    code_cell::clear_file_state_cache();

    // Find both positions
    let source = fs::read_to_string(&test_file.path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct CallFinder {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for CallFinder {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if let syn::Expr::Call(call) = node {
                use syn::spanned::Spanned;
                let span = call.func.span();
                let start = span.start();
                self.positions.push((start.line as u32, start.column as u32));
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut finder = CallFinder { positions: Vec::new() };
    finder.visit_file(&ast);

    assert_eq!(finder.positions.len(), 2, "Should find two calls");

    let (line1, col1) = finder.positions[0];
    let (line2, col2) = finder.positions[1];

    // Each location gets its own value
    let result1 = replace_me_at(10u32, test_file.path.to_str().unwrap(), line1, col1);
    let result2 = replace_me_at(20u32, test_file.path.to_str().unwrap(), line2, col2);

    assert_eq!(result1, 10u32);
    assert_eq!(result2, 20u32);

    env::remove_var("CODE_CELL_MODE");
}

#[test]
fn test_replace_me_with_vec() {
    // Test replacing with a more complex type
    let test_file = TestFile::new(
        r#"fn main() {
    let x = replace_me(vec![1, 2, 3]);
}
"#,
    );

    env::set_var("CODE_CELL_MODE", "write");

    // Clear any cached state
    code_cell::clear_file_state_cache();

    let (line, column) = find_call_position(&test_file.path);

    // Call replace_me_at with a vec
    let result = replace_me_at(vec![1i32, 2, 3], test_file.path.to_str().unwrap(), line, column);

    assert_eq!(result, vec![1i32, 2, 3]);

    // The file should contain the baked vec representation
    let content = test_file.read();
    assert!(
        !content.contains("replace_me"),
        "replace_me should be replaced. Content:\n{}",
        content
    );

    env::remove_var("CODE_CELL_MODE");
}
