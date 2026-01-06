/// Test that we detect and panic on concurrent modifications from other processes
use std::fs;

fn find_litter_positions(file_path: &std::path::Path) -> Vec<(u32, u32)> {
    use syn::visit::Visit;

    let source = std::fs::read_to_string(file_path).unwrap();
    let ast: syn::File = syn::parse_file(&source).unwrap();

    struct Finder {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for Finder {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if let syn::Expr::Call(call) = node {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(segment) = path.path.segments.last() {
                        if segment.ident == "literal" {
                            let span = segment.ident.span();
                            let start = span.start();
                            self.positions
                                .push((start.line as u32, start.column as u32));
                        }
                    }
                }
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut finder = Finder { positions: vec![] };
    finder.visit_file(&ast);
    finder.positions
}

#[test]
#[should_panic(expected = "CONCURRENT MODIFICATION DETECTED")]
fn test_detects_external_file_modification() {
    let test_code = r#"fn example() {
    let x = literal(42);
}
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("concurrent_test.rs");
    fs::write(&test_file, test_code).unwrap();

    // Load the file into inline's state
    let positions = find_litter_positions(&test_file);
    let _index =
        jeb_literal::runtime::get_macro_index(&test_file, positions[0].0, positions[0].1).unwrap();

    // Simulate another process modifying the file
    // (In reality, this would be a different process, but we can simulate it)
    let modified_code = r#"fn example() {
    let x = literal(999);
}
"#;
    fs::write(&test_file, modified_code).unwrap();

    // Now try to write - this should panic because the file was modified externally
    let new_tokens: proc_macro2::TokenStream = "100".parse().unwrap();
    jeb_literal::runtime::update_macro_by_index(&test_file, 0, new_tokens).unwrap();
    jeb_literal::runtime::write_to_disk(&test_file).unwrap(); // Should panic here
}

#[test]
fn test_no_panic_when_no_concurrent_modification() {
    let test_code = r#"fn example() {
    let x = literal(42);
}
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("normal_test.rs");
    fs::write(&test_file, test_code).unwrap();

    // Load the file
    let positions = find_litter_positions(&test_file);
    let _index =
        jeb_literal::runtime::get_macro_index(&test_file, positions[0].0, positions[0].1).unwrap();

    // Modify through inline - should work fine
    let new_tokens: proc_macro2::TokenStream = "100".parse().unwrap();
    jeb_literal::runtime::update_macro_by_index(&test_file, 0, new_tokens).unwrap();
    jeb_literal::runtime::write_to_disk(&test_file).unwrap(); // Should succeed

    // Verify the write happened
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("100"));
}

#[test]
fn test_multiple_writes_without_external_modification() {
    let test_code = r#"fn example() {
    let x = literal(42);
}
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("multi_write_test.rs");
    fs::write(&test_file, test_code).unwrap();

    // Load the file
    let positions = find_litter_positions(&test_file);
    let _index =
        jeb_literal::runtime::get_macro_index(&test_file, positions[0].0, positions[0].1).unwrap();

    // First write
    let tokens1: proc_macro2::TokenStream = "100".parse().unwrap();
    jeb_literal::runtime::update_macro_by_index(&test_file, 0, tokens1).unwrap();
    jeb_literal::runtime::write_to_disk(&test_file).unwrap();

    // Second write - should work because we track the disk state after the first write
    let tokens2: proc_macro2::TokenStream = "200".parse().unwrap();
    jeb_literal::runtime::update_macro_by_index(&test_file, 0, tokens2).unwrap();
    jeb_literal::runtime::write_to_disk(&test_file).unwrap();

    // Verify the final write
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("200"));
}
