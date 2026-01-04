//! Tests for default values in literal!() macro
//!
//! Verifies that literal!() with no arguments uses Default::default()

use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_empty_literal_uses_default() {
    // Test that literal!() with no arguments uses Default::default()

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Create a file with an empty literal!()
    let source = r#"fn test() {
    let x: jeb_literal::Literal<u32> = jeb_literal::literal!();
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "memory");

    // Find the literal position
    let positions = find_literal_positions(&path);
    assert_eq!(positions.len(), 1, "Should find exactly 1 literal");
    let (line, col) = positions[0];

    // Create the literal with type annotation (required for Default)
    let counter: jeb_literal::Literal<u32> =
        jeb_literal::Literal::__new(u32::default(), path.to_str().unwrap(), line, col);

    // Should start at 0 (default for u32)
    assert_eq!(*counter, 0u32, "Empty literal!() should use Default::default() (0 for u32)");

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_empty_literal_with_various_types() {
    // Test that Default works with different types

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let a: jeb_literal::Literal<u32> = jeb_literal::literal!();
    let b: jeb_literal::Literal<i32> = jeb_literal::literal!();
    let c: jeb_literal::Literal<String> = jeb_literal::literal!();
    let d: jeb_literal::Literal<Vec<u8>> = jeb_literal::literal!();
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "memory");

    let positions = find_literal_positions(&path);
    assert_eq!(positions.len(), 4, "Should find 4 literals");

    // Test u32
    let lit_u32: jeb_literal::Literal<u32> =
        jeb_literal::Literal::__new(u32::default(), path.to_str().unwrap(), positions[0].0, positions[0].1);
    assert_eq!(*lit_u32, 0u32);

    // Test i32
    let lit_i32: jeb_literal::Literal<i32> =
        jeb_literal::Literal::__new(i32::default(), path.to_str().unwrap(), positions[1].0, positions[1].1);
    assert_eq!(*lit_i32, 0i32);

    // Test String
    let lit_string: jeb_literal::Literal<String> =
        jeb_literal::Literal::__new(String::default(), path.to_str().unwrap(), positions[2].0, positions[2].1);
    assert_eq!(*lit_string, "");

    // Test Vec<u8>
    let lit_vec: jeb_literal::Literal<Vec<u8>> =
        jeb_literal::Literal::__new(Vec::default(), path.to_str().unwrap(), positions[3].0, positions[3].1);
    assert_eq!(*lit_vec, Vec::<u8>::new());

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_explicit_value_still_works() {
    // Ensure that providing an explicit value still works as before

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let x = jeb_literal::literal!(100u32);
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "memory");

    let positions = find_literal_positions(&path);
    let (line, col) = positions[0];

    let value = jeb_literal::Literal::__new(100u32, path.to_str().unwrap(), line, col);

    assert_eq!(*value, 100u32, "Explicit value should still work");

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_mixed_empty_and_explicit_literals() {
    // Test mixing empty and explicit literals in the same file

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let source = r#"fn test() {
    let a: jeb_literal::Literal<u32> = jeb_literal::literal!();
    let b = jeb_literal::literal!(100u32);
    let c: jeb_literal::Literal<String> = jeb_literal::literal!();
    let d = jeb_literal::literal!("hello".to_string());
}
"#;
    fs::write(&path, source).unwrap();

    env::set_var("LITERAL_MODE", "memory");

    let positions = find_literal_positions(&path);
    assert_eq!(positions.len(), 4);

    // Empty u32
    let lit_a: jeb_literal::Literal<u32> =
        jeb_literal::Literal::__new(u32::default(), path.to_str().unwrap(), positions[0].0, positions[0].1);
    assert_eq!(*lit_a, 0u32);

    // Explicit u32
    let lit_b = jeb_literal::Literal::__new(100u32, path.to_str().unwrap(), positions[1].0, positions[1].1);
    assert_eq!(*lit_b, 100u32);

    // Empty String
    let lit_c: jeb_literal::Literal<String> =
        jeb_literal::Literal::__new(String::default(), path.to_str().unwrap(), positions[2].0, positions[2].1);
    assert_eq!(*lit_c, "");

    // Explicit String
    let lit_d = jeb_literal::Literal::__new("hello".to_string(), path.to_str().unwrap(), positions[3].0, positions[3].1);
    assert_eq!(*lit_d, "hello");

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
