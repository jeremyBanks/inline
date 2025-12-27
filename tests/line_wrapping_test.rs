use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_long_vec_causes_line_wrapping() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Start with SHORT values
    let original = r#"fn main() {
    let a = litter!(vec![1u32, 2u32]);
    let b = litter!(100u32);
}
"#;
    fs::write(&path, original).unwrap();
    println!("=== ORIGINAL ===");
    println!("{}", original);

    env::set_var("LITTER_UPDATE", "1");

    let positions = find_all_positions(&path);
    let (a_line, a_col) = positions[0];
    let (b_line, b_col) = positions[1];

    println!("Initial positions:");
    println!("  a: line {}, col {}", a_line, a_col);
    println!("  b: line {}, col {}", b_line, b_col);

    // Create litter for the vector
    let mut litter_a = litter::Litter::__new(
        vec![1u32, 2u32],
        path.to_str().unwrap(),
        a_line,
        a_col,
    );

    // Update to a REALLY LONG vector that will definitely wrap
    let long_vec: Vec<u32> = (0..100).collect();
    println!("\n=== UPDATING A TO LONG VECTOR (0..100) ===");
    litter_a.set(long_vec);

    let content = fs::read_to_string(&path).unwrap();
    println!("{}", content);

    // Count lines in the file
    let line_count = content.lines().count();
    println!("\nFile now has {} lines (was 4)", line_count);

    // Find positions after update
    let positions_after = find_all_positions(&path);
    println!("\nPositions after update:");
    println!("  a: line {}, col {}", positions_after[0].0, positions_after[0].1);
    println!("  b: line {}, col {}", positions_after[1].0, positions_after[1].1);

    if positions_after[1].0 != b_line {
        println!("\n❌ BOOM! B moved from line {} to line {}", b_line, positions_after[1].0);
        println!("This means the next update to B will FAIL because we're looking at the wrong line!");

        // Try to update B using the OLD position
        let mut litter_b = litter::Litter::__new(
            100u32,
            path.to_str().unwrap(),
            b_line,  // OLD position
            b_col,
        );

        println!("\n=== ATTEMPTING TO UPDATE B USING OLD POSITION ===");
        litter_b.set(999u32);

        // Check if it worked
        let final_content = fs::read_to_string(&path).unwrap();
        if final_content.contains("litter!(999u32)") {
            println!("Somehow it worked?");
        } else {
            println!("Failed to update B! The file still has:");
            println!("{}", final_content);
        }

        panic!("Line numbers shifted - approach is broken!");
    } else {
        println!("\n✓ Surprisingly, B stayed at line {}", b_line);
    }

    env::remove_var("LITTER_UPDATE");
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
                segment.ident == "litter"
            } else {
                false
            };

            if is_litter {
                let span = node.mac.path.segments.last().unwrap().ident.span();
                let start = span.start();
                self.positions.push((start.line as u32, start.column as u32));
            }
            syn::visit::visit_expr_macro(self, node);
        }
    }

    let mut collector = MacroCollector { positions: Vec::new() };
    collector.visit_file(&ast);
    collector.positions
}
