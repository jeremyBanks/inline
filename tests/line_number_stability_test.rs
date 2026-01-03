use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_line_number_stability_with_multiple_litters() {
    // This tests the REAL concern: does updating the first inline
    // affect the line number of the second inline?

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let original = r#"fn main() {
    let a = inline!(1u32);
    let b = inline!(2u32);
    let c = inline!(3u32);
}
"#;
    fs::write(&path, original).unwrap();
    println!("=== ORIGINAL FILE ===");
    println!("{}", original);

    env::set_var("INLINE_MODE", "write");

    // Find all positions initially
    let positions = find_all_positions(&path);
    println!("\n=== INITIAL POSITIONS ===");
    for (i, (line, col)) in positions.iter().enumerate() {
        println!("Inline {}: line {}, column {}", i, line, col);
    }

    assert_eq!(positions.len(), 3, "Should find 3 litters");
    let (a_line, a_col) = positions[0];
    let (b_line, b_col) = positions[1];
    let (c_line, c_col) = positions[2];

    // Create inline instances
    let mut litter_a = inline::Inline::__new(1u32, path.to_str().unwrap(), a_line, a_col);
    let mut litter_b = inline::Inline::__new(2u32, path.to_str().unwrap(), b_line, b_col);
    let mut litter_c = inline::Inline::__new(3u32, path.to_str().unwrap(), c_line, c_col);

    // Update A
    println!("\n=== UPDATING A (1 -> 999) ===");
    litter_a.set(999u32);
    println!("{}", fs::read_to_string(&path).unwrap());

    // Check positions after updating A
    let positions_after_a = find_all_positions(&path);
    println!("Positions after updating A:");
    for (i, (line, col)) in positions_after_a.iter().enumerate() {
        println!("Inline {}: line {}, column {}", i, line, col);
    }

    // THE KEY QUESTION: Did B and C's line numbers change?
    assert_eq!(
        positions_after_a[1].0, b_line,
        "B's line number should NOT change after updating A"
    );
    assert_eq!(
        positions_after_a[2].0, c_line,
        "C's line number should NOT change after updating A"
    );

    // Now update B
    println!("\n=== UPDATING B (2 -> 888) ===");
    litter_b.set(888u32);
    println!("{}", fs::read_to_string(&path).unwrap());

    let positions_after_b = find_all_positions(&path);
    println!("Positions after updating B:");
    for (i, (line, col)) in positions_after_b.iter().enumerate() {
        println!("Inline {}: line {}, column {}", i, line, col);
    }

    // Check C's position
    assert_eq!(
        positions_after_b[2].0, c_line,
        "C's line number should NOT change after updating B"
    );

    // Finally update C
    println!("\n=== UPDATING C (3 -> 777) ===");
    litter_c.set(777u32);
    println!("{}", fs::read_to_string(&path).unwrap());

    let positions_final = find_all_positions(&path);
    println!("Final positions:");
    for (i, (line, col)) in positions_final.iter().enumerate() {
        println!("Inline {}: line {}, column {}", i, line, col);
    }

    // All positions should remain unchanged
    assert_eq!(
        positions_final[0].0, a_line,
        "A's line should not have changed"
    );
    assert_eq!(
        positions_final[1].0, b_line,
        "B's line should not have changed"
    );
    assert_eq!(
        positions_final[2].0, c_line,
        "C's line should not have changed"
    );

    println!("\n✓ ALL LINE NUMBERS REMAINED STABLE!");

    env::remove_var("INLINE_MODE");
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
                segment.ident == "inline"
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
