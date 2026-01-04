use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test file with Rust source code
struct TestFile {
    _dir: TempDir,
    pub path: PathBuf,
}

impl TestFile {
    fn new(content: &str) -> Self {
        Self::with_name(content, "test.rs")
    }

    fn with_name(content: &str, filename: &str) -> Self {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(filename);
        fs::write(&path, content).unwrap();
        TestFile { _dir: dir, path }
    }

    fn read(&self) -> String {
        fs::read_to_string(&self.path).unwrap()
    }

    fn contains(&self, s: &str) -> bool {
        self.read().contains(s)
    }

    fn assert_contains(&self, s: &str) {
        assert!(
            self.contains(s),
            "Expected file to contain: {}\n\nActual content:\n{}",
            s,
            self.read()
        );
    }

    fn assert_does_not_contain(&self, s: &str) {
        assert!(
            !self.contains(s),
            "Expected file to NOT contain: {}\n\nActual content:\n{}",
            s,
            self.read()
        );
    }
}

/// Helper to find all literal! macro positions in a file
fn find_litter_positions(file_path: &std::path::Path) -> Vec<(u32, u32)> {
    let source = fs::read_to_string(file_path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct MacroCollector {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for MacroCollector {
        fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
            // Check if this is a literal macro (might be just "inline" or "inline::inline")
            let is_litter = if let Some(segments) = node.mac.path.segments.iter().last() {
                segments.ident == "literal"
            } else {
                false
            };

            if is_litter {
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

#[test]
fn test_span_preservation() {
    // Test that syn preserves line/column information when parsing
    let source = r#"fn main() {
    let x = literal!(42);
}"#;

    let ast = syn::parse_file(source).unwrap();

    // Find the macro using syn::visit
    use syn::visit::Visit;

    struct MacroFinder {
        found_at: Option<(usize, usize)>,
    }

    impl<'ast> Visit<'ast> for MacroFinder {
        fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
            if let Some(ident) = node.mac.path.get_ident() {
                if ident == "literal" {
                    let span = ident.span();
                    let start = span.start();
                    self.found_at = Some((start.line, start.column));
                }
            }
        }
    }

    let mut finder = MacroFinder { found_at: None };
    finder.visit_file(&ast);

    // The macro should be at line 2 (1-indexed), some column
    assert!(finder.found_at.is_some(), "Should find the literal macro");
    let (line, _col) = finder.found_at.unwrap();
    assert_eq!(line, 2, "Macro should be on line 2");
}

#[test]
fn test_update_source_file() {
    // Test the low-level update_source_file function
    let test_file = TestFile::new(
        r#"fn main() {
    let x = literal!(42u32);
}
"#,
    );

    // Enable update mode
    env::set_var("LITERAL_MODE", "write");

    // Find the actual position of the macro
    let positions = find_litter_positions(&test_file.path);
    assert_eq!(positions.len(), 1, "Should find exactly one literal macro");
    let (line, column) = positions[0];

    // Update the value
    let new_tokens: proc_macro2::TokenStream = "100u32".parse().unwrap();
    jeb_literal::update_source_file(&test_file.path, line, column, new_tokens).unwrap();

    // Verify the file was updated
    test_file.assert_contains("literal!(100u32)");
    test_file.assert_does_not_contain("literal!(42u32)");

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_litter_basic_update() {
    // Test using the actual Inline API
    let test_file = TestFile::with_name(
        r#"#[allow(unused)]
fn test() {
    let x = inline::literal!(42u32);
}
"#,
        "test_litter_basic_update.rs",
    );

    env::set_var("LITERAL_MODE", "write");

    // Find the actual position
    let positions = find_litter_positions(&test_file.path);
    assert_eq!(positions.len(), 1, "Should find exactly one literal macro");
    let (line, column) = positions[0];

    // Create a Inline instance manually (simulating what the macro does)
    {
        let mut value = jeb_literal::Literal::__new(42u32, test_file.path.to_str().unwrap(), line, column);

        // Update the value
        value.value = 100u32;

        // Check that the value changed in memory
        assert_eq!(*value.get(), 100u32);
        // Drop happens here - triggers write
    }

    // Check that the file was updated
    test_file.assert_contains("literal!(100u32)");

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_litter_no_update_in_memory_mode() {
    let test_file = TestFile::new(
        r#"fn test() {
    let x = inline::literal!(42u32);
}
"#,
    );

    // Explicitly set memory mode (changes in memory only, no disk writes)
    env::set_var("LITERAL_MODE", "memory");

    let positions = find_litter_positions(&test_file.path);
    assert_eq!(positions.len(), 1);
    let (line, column) = positions[0];

    let mut value = jeb_literal::Literal::__new(42u32, test_file.path.to_str().unwrap(), line, column);

    // Update the value
    value.value = 100u32;

    // Value should change in memory
    assert_eq!(*value.get(), 100u32);

    // But file should NOT be updated (still contains original)
    test_file.assert_contains("literal!(42u32)");

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_databake_integration() {
    // Test that databake types work - just check that bake() produces something
    use databake::Bake;

    let value = vec![1u32, 2, 3];
    let env = databake::CrateEnv::default();
    let baked = value.bake(&env);

    // Should produce valid Rust code
    let baked_str = baked.to_string();
    // Just check it's not empty and contains numbers
    assert!(!baked_str.is_empty(), "Baked value should not be empty");
    assert!(
        baked_str.contains('1') && baked_str.contains('2') && baked_str.contains('3'),
        "Baked value should contain the numbers: {}",
        baked_str
    );
}

#[test]
fn test_multiple_litters_in_same_file() {
    let test_file = TestFile::new(
        r#"fn test() {
    let a = inline::literal!(1u32);
    let b = inline::literal!(2u32);
    let c = inline::literal!(3u32);
}
"#,
    );

    env::set_var("LITERAL_MODE", "write");

    // Find all positions
    let positions = find_litter_positions(&test_file.path);
    assert_eq!(positions.len(), 3, "Should find 3 macros");

    // Create inline instances for each
    {
        let mut litter_a = jeb_literal::Literal::__new(
            1u32,
            test_file.path.to_str().unwrap(),
            positions[0].0,
            positions[0].1,
        );

        let mut litter_b = jeb_literal::Literal::__new(
            2u32,
            test_file.path.to_str().unwrap(),
            positions[1].0,
            positions[1].1,
        );

        let mut litter_c = jeb_literal::Literal::__new(
            3u32,
            test_file.path.to_str().unwrap(),
            positions[2].0,
            positions[2].1,
        );

        // Update them in different orders
        litter_b.value = 20u32;
        litter_a.value = 10u32;
        litter_c.value = 30u32;
        // All values drop here - triggers writes
    }

    // All updates should have persisted
    let content = test_file.read();
    assert!(
        content.contains("literal!(10u32)"),
        "Should contain updated a"
    );
    assert!(
        content.contains("literal!(20u32)"),
        "Should contain updated b"
    );
    assert!(
        content.contains("literal!(30u32)"),
        "Should contain updated c"
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_litter_no_change_optimization() {
    let test_file = TestFile::new(
        r#"fn test() {
    let x = inline::literal!(42u32);
}
"#,
    );

    env::set_var("LITERAL_MODE", "write");

    let positions = find_litter_positions(&test_file.path);
    let (line, column) = positions[0];

    let mut value = jeb_literal::Literal::__new(42u32, test_file.path.to_str().unwrap(), line, column);

    // Set to the same value
    value.value = 42u32;

    // Value should still be 42
    assert_eq!(*value.get(), 42u32);

    // File should still contain original value
    test_file.assert_contains("literal!(42u32)");

    env::remove_var("LITERAL_MODE");
}
