use std::env;
use std::fs;
use tempfile::TempDir;
use jeb_literal::LiteralPrivate;

#[test]
fn test_long_vec_causes_line_wrapping() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Start with SHORT values
    let original = r#"fn main() {
    let a = literal!(vec![1u32, 2u32]);
    let b = literal!(100u32);
}
"#;
    fs::write(&path, original).unwrap();
    println!("=== ORIGINAL ===");
    println!("{}", original);

    env::set_var("LITERAL_MODE", "write");

    let positions = find_all_positions(&path);
    let (a_line, a_col) = positions[0];
    let (b_line, b_col) = positions[1];

    println!("Initial positions:");
    println!("  a: line {}, col {}", a_line, a_col);
    println!("  b: line {}, col {}", b_line, b_col);

    // Create inline for the vector
    let mut litter_a =
        jeb_literal::Literal::__new(vec![1u32, 2u32], path.to_str().unwrap(), a_line, a_col);

    // Update to a REALLY LONG vector that will definitely wrap
    let long_vec: Vec<u32> = (0..100).collect();
    println!("\n=== UPDATING A TO LONG VECTOR (0..100) ===");
    litter_a.literal = long_vec;

    let content = fs::read_to_string(&path).unwrap();
    println!("{}", content);

    // Count lines in the file
    let line_count = content.lines().count();
    println!("\nFile now has {} lines (was 4)", line_count);

    // Find positions after update
    let positions_after = find_all_positions(&path);
    println!("\nPositions after update:");
    println!(
        "  a: line {}, col {}",
        positions_after[0].0, positions_after[0].1
    );
    println!(
        "  b: line {}, col {}",
        positions_after[1].0, positions_after[1].1
    );

    if positions_after[1].0 != b_line {
        println!(
            "\n✓ As expected, B moved from line {} to line {}",
            b_line, positions_after[1].0
        );
        println!("This would break a position-based approach, but index-based should handle it!");

        // Try to update B using the OLD position - this tests the index-based approach
        {
            let mut litter_b = jeb_literal::Literal::__new(
                100u32,
                path.to_str().unwrap(),
                b_line, // OLD position from initial parse
                b_col,
            );

            println!("\n=== UPDATING B USING ORIGINAL POSITION (index-based lookup) ===");
            litter_b.literal = 999u32;
            // Drop happens here - triggers write
        }

        // Check if it worked
        let final_content = fs::read_to_string(&path).unwrap();
        println!("\n=== FINAL FILE CONTENT ===");
        println!("{}", final_content);

        if final_content.contains("literal!(999u32)") {
            println!("\n✅ SUCCESS! Index-based approach works even when line numbers shift!");
            println!(
                "The original position (line {}) was mapped to a stable index,",
                b_line
            );
            println!(
                "which correctly found B even though it's now at line {}",
                positions_after[1].0
            );
        } else {
            println!("❌ Failed to update B! Expected to find literal!(999u32)");
            panic!("Index-based approach failed!");
        }
    } else {
        println!("\n✓ B stayed at line {} (formatter didn't wrap)", b_line);
        println!("Let's still verify the index-based approach works by updating B");

        {
            let mut litter_b = jeb_literal::Literal::__new(100u32, path.to_str().unwrap(), b_line, b_col);

            litter_b.literal = 999u32;
            // Drop happens here - triggers write
        }

        let final_content = fs::read_to_string(&path).unwrap();
        assert!(
            final_content.contains("literal!(999u32)"),
            "Should update B successfully"
        );
        println!("✅ B updated successfully");
    }

    env::remove_var("LITERAL_MODE");
}

fn find_all_positions(path: &std::path::Path) -> Vec<(u32, u32)> {
    let source = fs::read_to_string(path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct MacroCollector {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for MacroCollector {
        fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
            let is_litter = if let Some(segment) = node.mac.path.segments.last() {
                segment.ident == "literal"
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
