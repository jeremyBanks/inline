use std::env;
use std::fs;
use tempfile::TempDir;
use code_cell::CodeCellPrivate;

/// Helper to find all code_cell() call positions in a file
fn find_litter_positions(file_path: &std::path::Path) -> Vec<(u32, u32)> {
    let source = fs::read_to_string(file_path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct CallCollector {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for CallCollector {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if let syn::Expr::Call(call) = node {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(segment) = path.path.segments.last() {
                        if segment.ident == "code_cell" {
                            let span = segment.ident.span();
                            let start = span.start();
                            self.positions.push((start.line as u32, start.column as u32));
                        }
                    }
                }
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut collector = CallCollector {
        positions: Vec::new(),
    };
    collector.visit_file(&ast);
    collector.positions
}

#[test]
fn test_verify_mode_matching_value() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Write a file with a inline value
    let content = r#"fn test() {
    let x = code_cell::code_cell(42u32);
}
"#;
    fs::write(&path, content).unwrap();

    // Enable verify mode
    env::set_var("CODE_CELL_MODE", "verify");

    let positions = find_litter_positions(&path);
    let (line, column) = positions[0];

    let mut value = code_cell::CodeCell::__new(42u32, path.to_str().unwrap(), line, column);

    // Setting to the same value should succeed in verify mode
    value.value = 42u32;

    env::remove_var("CODE_CELL_MODE");
}

#[test]
#[should_panic(expected = "CodeCell verification failed")]
fn test_verify_mode_mismatched_value() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Write a file with a inline value
    let content = r#"fn test() {
    let x = code_cell::code_cell(42u32);
}
"#;
    fs::write(&path, content).unwrap();

    // Enable verify mode
    env::set_var("CODE_CELL_MODE", "verify");

    let positions = find_litter_positions(&path);
    let (line, column) = positions[0];

    {
        let mut value = code_cell::CodeCell::__new(42u32, path.to_str().unwrap(), line, column);

        // Setting to a different value should panic in verify mode
        value.value = 100u32;
        // Drop happens here - should panic due to verification failure
    }

    env::remove_var("CODE_CELL_MODE");
}

#[test]
fn test_verify_mode_complex_value() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Write a file with a complex inline value
    let content = r#"fn test() {
    let x = code_cell::code_cell(vec![1u32, 2u32, 3u32]);
}
"#;
    fs::write(&path, content).unwrap();

    // Enable verify mode
    env::set_var("CODE_CELL_MODE", "verify");

    let positions = find_litter_positions(&path);
    let (line, column) = positions[0];

    let mut value =
        code_cell::CodeCell::__new(vec![1u32, 2u32, 3u32], path.to_str().unwrap(), line, column);

    // Setting to the same value should succeed
    value.value = vec![1u32, 2u32, 3u32];

    env::remove_var("CODE_CELL_MODE");
}

#[test]
#[should_panic(expected = "CodeCell verification failed")]
fn test_verify_mode_complex_value_mismatch() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Write a file with a complex inline value
    let content = r#"fn test() {
    let x = code_cell::code_cell(vec![1u32, 2u32, 3u32]);
}
"#;
    fs::write(&path, content).unwrap();

    // Enable verify mode
    env::set_var("CODE_CELL_MODE", "verify");

    let positions = find_litter_positions(&path);
    let (line, column) = positions[0];

    {
        let mut value =
            code_cell::CodeCell::__new(vec![1u32, 2u32, 3u32], path.to_str().unwrap(), line, column);

        // Setting to a different value should panic
        value.value = vec![1u32, 2u32, 3u32, 4u32];
        // Drop happens here - should panic due to verification failure
    }

    env::remove_var("CODE_CELL_MODE");
}
