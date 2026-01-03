use std::sync::Arc;
use std::thread;

fn find_litter_positions(file_path: &std::path::Path) -> Vec<(u32, u32)> {
    use syn::visit::Visit;

    let source = std::fs::read_to_string(file_path).unwrap();
    let ast: syn::File = syn::parse_file(&source).unwrap();

    struct Finder {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for Finder {
        fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
            if let Some(segment) = node.mac.path.segments.last() {
                if segment.ident == "inline" {
                    let start = segment.ident.span().start();
                    self.positions
                        .push((start.line as u32, start.column as u32));
                }
            }
            syn::visit::visit_expr_macro(self, node);
        }

        fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
            if let Some(segment) = node.mac.path.segments.last() {
                if segment.ident == "inline" {
                    let start = segment.ident.span().start();
                    self.positions
                        .push((start.line as u32, start.column as u32));
                }
            }
            syn::visit::visit_stmt_macro(self, node);
        }
    }

    let mut finder = Finder { positions: vec![] };
    finder.visit_file(&ast);
    finder.positions
}

#[test]
fn test_multi_threaded_access() {
    // Create a simple test file with multiple inline! macros
    let test_code = r#"fn example() {
    let a = inline!(42);
    let b = inline!(100);
    let c = inline!(200);
}
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("multi_thread_test.rs");
    std::fs::write(&test_file, test_code).unwrap();

    // Find actual positions
    let positions = find_litter_positions(&test_file);
    assert_eq!(positions.len(), 3);

    // Verify initial parsing works using actual positions
    let index_0 =
        inline::runtime::get_macro_index(&test_file, positions[0].0, positions[0].1).unwrap();
    let index_1 =
        inline::runtime::get_macro_index(&test_file, positions[1].0, positions[1].1).unwrap();
    let index_2 =
        inline::runtime::get_macro_index(&test_file, positions[2].0, positions[2].1).unwrap();

    assert_eq!(index_0, 0);
    assert_eq!(index_1, 1);
    assert_eq!(index_2, 2);

    // Test concurrent reads from multiple threads
    let test_file = Arc::new(test_file);
    let handles: Vec<_> = (0..4)
        .map(|thread_id| {
            let test_file = Arc::clone(&test_file);
            thread::spawn(move || {
                // Each thread reads all three macros
                for i in 0..3 {
                    let tokens = inline::runtime::get_macro_tokens_by_index(&test_file, i).unwrap();
                    assert!(!tokens.is_empty());
                }
                thread_id
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Test that modifications are visible across threads
    // Spawn a thread that modifies macro 0
    let test_file_clone = Arc::clone(&test_file);
    let modifier = thread::spawn(move || {
        let new_tokens: proc_macro2::TokenStream = "999".parse().unwrap();
        inline::runtime::update_macro_by_index(&test_file_clone, 0, new_tokens).unwrap();
        inline::runtime::write_to_disk(&test_file_clone).unwrap();
    });

    modifier.join().unwrap();

    // Verify from main thread that modification is visible
    let tokens = inline::runtime::get_macro_tokens_by_index(&test_file, 0).unwrap();
    assert_eq!(tokens.to_string(), "999");

    // Spawn another thread to verify it sees the update
    let test_file_clone = Arc::clone(&test_file);
    let reader = thread::spawn(move || {
        let tokens = inline::runtime::get_macro_tokens_by_index(&test_file_clone, 0).unwrap();
        assert_eq!(tokens.to_string(), "999");
    });

    reader.join().unwrap();
}

#[test]
fn test_sequential_modifications_across_threads() {
    let test_code = r#"fn example() {
    let x = inline!(0);
}
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("sequential_test.rs");
    std::fs::write(&test_file, test_code).unwrap();

    let test_file = Arc::new(test_file);

    // Find actual position and get initial index
    let positions = find_litter_positions(&test_file);
    inline::runtime::get_macro_index(&test_file, positions[0].0, positions[0].1).unwrap();

    // Spawn threads that each increment the value sequentially
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let test_file = Arc::clone(&test_file);
            thread::spawn(move || {
                // Each thread waits a bit to stagger modifications
                thread::sleep(std::time::Duration::from_millis(i * 10));

                let new_value = format!("{}", (i + 1) * 100);
                let new_tokens: proc_macro2::TokenStream = new_value.parse().unwrap();
                inline::runtime::update_macro_by_index(&test_file, 0, new_tokens).unwrap();
                inline::runtime::write_to_disk(&test_file).unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // After all modifications, verify we can still read
    let final_tokens = inline::runtime::get_macro_tokens_by_index(&test_file, 0).unwrap();
    // Should be 500 (the last write wins)
    assert_eq!(final_tokens.to_string(), "500");
}
