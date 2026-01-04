//! Tests for DerefMut + write-on-drop functionality
//!
//! Verifies that mutating through DerefMut triggers automatic writes on drop

use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_deref_mut_triggers_write_on_drop() {
    // Test that mutating a value through DerefMut causes it to be written on drop

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let mut counter = jeb_literal::literal!(0u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "write");

    let positions = find_literal_positions(&path);
    let (line, col) = positions[0];

    {
        // Create a mutable literal
        let mut counter = jeb_literal::Literal::__new(0u32, path.to_str().unwrap(), line, col);

        // Mutate through DerefMut
        *counter += 1;

        assert_eq!(*counter, 1u32, "Value should be mutated in memory");
        // Drop happens here - should trigger write
    }

    // Verify file was updated
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("literal!(1u32)"),
        "File should contain updated value after drop. Actual:\n{}",
        content
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_deref_mut_no_write_if_unchanged() {
    // Test that if value isn't actually mutated, no write occurs

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let mut x = jeb_literal::literal!(42u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "write");

    let positions = find_literal_positions(&path);
    let (line, col) = positions[0];

    {
        let x = jeb_literal::Literal::__new(42u32, path.to_str().unwrap(), line, col);

        // Access but don't modify
        let _val = *x;

        // Drop - should NOT trigger write because value unchanged
    }

    // File should still contain original value
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("literal!(42u32)"),
        "File should still contain original value"
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_deref_mut_with_complex_mutation() {
    // Test DerefMut with more complex mutations (String)

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let mut s = jeb_literal::literal!("hello".to_string());
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "write");

    let positions = find_literal_positions(&path);
    let (line, col) = positions[0];

    {
        let mut s = jeb_literal::Literal::__new(
            "hello".to_string(),
            path.to_str().unwrap(),
            line,
            col,
        );

        // Mutate the string
        s.push_str(" world");

        assert_eq!(*s, "hello world");
        // Drop - triggers write
    }

    // Verify file was updated
    let content = fs::read_to_string(&path).unwrap();
    // databake may format this differently, just check the value is there
    assert!(
        content.contains("hello world"),
        "File should contain updated string value. Actual:\n{}",
        content
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_deref_mut_with_vec() {
    // Test DerefMut with Vec mutations

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let mut v = jeb_literal::literal!(vec![1u32, 2u32]);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "write");

    let positions = find_literal_positions(&path);
    let (line, col) = positions[0];

    {
        let mut v = jeb_literal::Literal::__new(
            vec![1u32, 2u32],
            path.to_str().unwrap(),
            line,
            col,
        );

        // Mutate the vec
        v.push(3u32);

        assert_eq!(*v, vec![1u32, 2u32, 3u32]);
        // Drop - triggers write
    }

    // Verify file was updated
    let content = fs::read_to_string(&path).unwrap();
    // databake may format vec differently, just check all values are present
    assert!(
        content.contains("1u32") && content.contains("2u32") && content.contains("3u32"),
        "File should contain all vec elements. Actual:\n{}",
        content
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_value_field_assignment_works_with_deref_mut() {
    // Test that .value field assignment works alongside DerefMut

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let mut counter = jeb_literal::literal!(0u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "write");

    let positions = find_literal_positions(&path);
    let (line, col) = positions[0];

    {
        let mut counter = jeb_literal::Literal::__new(0u32, path.to_str().unwrap(), line, col);

        // Use direct .value field assignment
        counter.value = 10u32;

        assert_eq!(*counter, 10u32);
        assert_eq!(counter.value, 10u32);
        // Drop - triggers write with new value
    }

    // File should contain the new value
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("literal!(10u32)"),
        "File should contain value from .value assignment"
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_deref_mut_memory_mode() {
    // Test that DerefMut works in memory mode (mutations work in memory)

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let mut counter = jeb_literal::literal!(0u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "memory");

    let positions = find_literal_positions(&path);
    let (line, col) = positions[0];

    {
        let mut counter = jeb_literal::Literal::__new(0u32, path.to_str().unwrap(), line, col);

        // Mutate through DerefMut
        *counter += 5;

        // Verify mutation worked in memory
        assert_eq!(*counter, 5u32, "Mutation through DerefMut should work in memory mode");
        // Drop in memory mode - should not write to file (but we can't reliably test this
        // due to parallel test execution affecting env vars)
    }

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_multiple_mutations_before_drop() {
    // Test multiple mutations through DerefMut before drop

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let mut counter = jeb_literal::literal!(0u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "write");

    let positions = find_literal_positions(&path);
    let (line, col) = positions[0];

    {
        let mut counter = jeb_literal::Literal::__new(0u32, path.to_str().unwrap(), line, col);

        // Multiple mutations
        *counter += 1;
        *counter += 2;
        *counter *= 3;

        assert_eq!(*counter, 9u32);  // (0 + 1 + 2) * 3 = 9
        // Drop - should write final value
    }

    // File should contain final value
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("literal!(9u32)"),
        "File should contain final mutated value. Actual:\n{}",
        content
    );

    env::remove_var("LITERAL_MODE");
}

/// Helper to find all literal! macro positions in a file
fn find_literal_positions(path: &std::path::Path) -> Vec<(u32, u32)> {
    let source = fs::read_to_string(path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct MacroCollector {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for MacroCollector {
        fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
            let is_literal = if let Some(segment) = node.mac.path.segments.last() {
                segment.ident == "literal"
            } else {
                false
            };

            if is_literal {
                let span = node.mac.path.segments.last().unwrap().ident.span();
                let start = span.start();
                self.positions
                    .push((start.line as u32, start.column as u32));
            }
            syn::visit::visit_expr_macro(self, node);
        }
    }

    let mut collector = MacroCollector {
        positions: Vec::new(),
    };
    collector.visit_file(&ast);
    collector.positions
}
