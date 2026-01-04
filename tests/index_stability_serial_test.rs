//! Tests for registry key stability using index-based lookups
//!
//! These tests verify that literal values persist across line insertions,
//! which is the critical behavior enabled by index-based registry keys.

use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_value_persists_across_line_insertions() {
    // CRITICAL TEST: Verify that inserting lines ABOVE a literal
    // doesn't reset its value (the bug we just fixed)

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Original file with a single literal at line 2
    let original = r#"fn main() {
    let counter = literal!(0u32);
    println!("Counter: {}", *counter);
}
"#;
    fs::write(&path, original).unwrap();

    env::set_var("LITERAL_MODE", "write");

    // Find the literal's initial position
    let positions = find_all_literal_positions(&path);
    assert_eq!(positions.len(), 1, "Should find exactly 1 literal");
    let (initial_line, initial_col) = positions[0];

    println!("\n=== INITIAL STATE ===");
    println!("File path: {}", path.display());
    println!("Literal at line {}, column {}", initial_line, initial_col);
    println!("{}", original);

    // Create the literal and set it to a new value
    let mut counter = jeb_literal::Literal::__new(
        0u32,
        path.to_str().unwrap(),
        initial_line,
        initial_col,
    );

    println!("\n=== SETTING VALUE TO 42 ===");
    counter.value = 42u32;
    drop(counter); // Release the lock

    // Verify the file was updated
    let after_set = fs::read_to_string(&path).unwrap();
    println!("{}", after_set);
    assert!(
        after_set.contains("literal!(42u32)"),
        "File should contain the updated value"
    );

    // Now insert 10 lines ABOVE the literal
    // This changes the literal's line number from ~2 to ~12
    println!("\n=== INSERTING 10 LINES ABOVE LITERAL ===");
    let modified = r#"// This is a comment
// Line 2
// Line 3
// Line 4
// Line 5
// Line 6
// Line 7
// Line 8
// Line 9
// Line 10
fn main() {
    let counter = literal!(42u32);
    println!("Counter: {}", *counter);
}
"#;
    fs::write(&path, modified).unwrap();

    // Clear the file state cache so the runtime re-parses the file
    jeb_literal::clear_file_state_cache();

    println!("{}", modified);

    // Find the new position
    let new_positions = find_all_literal_positions(&path);
    assert_eq!(new_positions.len(), 1, "Should still find exactly 1 literal");
    let (new_line, new_col) = new_positions[0];

    println!("\n=== AFTER LINE INSERTION ===");
    println!("Literal now at line {}, column {}", new_line, new_col);
    println!(
        "Line number changed: {} -> {}",
        initial_line, new_line
    );

    // THE CRITICAL TEST: Access the literal at its NEW position
    // With index-based keys, this should resolve to the SAME registry entry
    // and return our value of 42, not reset to the initial value of 0
    println!("\n=== ACCESSING LITERAL AT NEW POSITION ===");
    let counter_after_shift =
        jeb_literal::Literal::__new(0u32, path.to_str().unwrap(), new_line, new_col);

    let value = *counter_after_shift;
    println!("Value after line shift: {}", value);

    assert_eq!(
        value, 42,
        "CRITICAL: Value should persist across line insertions! \
         Expected 42 (the value we set), but got {} (likely the initial value). \
         This means the registry key changed when lines shifted.",
        value
    );

    println!("\n✓ SUCCESS: Value persisted across line insertion!");
    println!("  The literal moved from line {} to line {}", initial_line, new_line);
    println!("  But its value remained 42 (not reset to 0)");
    println!("  This proves index-based registry keys are working!");

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_multiple_literals_maintain_distinct_identities() {
    // Test that when we insert lines between literals,
    // each literal maintains its own unique identity and value

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let original = r#"fn main() {
    let a = literal!(10u32);
    let b = literal!(20u32);
    let c = literal!(30u32);
}
"#;
    fs::write(&path, original).unwrap();

    env::set_var("LITERAL_MODE", "write");

    // Find all three literals
    let positions = find_all_literal_positions(&path);
    assert_eq!(positions.len(), 3, "Should find 3 literals");

    let (a_line, a_col) = positions[0];
    let (b_line, b_col) = positions[1];
    let (c_line, c_col) = positions[2];

    println!("\n=== INITIAL POSITIONS ===");
    println!("A: line {}, col {}", a_line, a_col);
    println!("B: line {}, col {}", b_line, b_col);
    println!("C: line {}, col {}", c_line, c_col);

    // Create and set all three literals to different values
    let mut lit_a = jeb_literal::Literal::__new(10u32, path.to_str().unwrap(), a_line, a_col);
    let mut lit_b = jeb_literal::Literal::__new(20u32, path.to_str().unwrap(), b_line, b_col);
    let mut lit_c = jeb_literal::Literal::__new(30u32, path.to_str().unwrap(), c_line, c_col);

    println!("\n=== SETTING UNIQUE VALUES ===");
    lit_a.value = 111u32;
    lit_b.value = 222u32;
    lit_c.value = 333u32;
    drop(lit_a);
    drop(lit_b);
    drop(lit_c);

    // Insert lines between A and B
    println!("\n=== INSERTING LINES BETWEEN A AND B ===");
    let modified = r#"fn main() {
    let a = literal!(111u32);
    // Extra line 1
    // Extra line 2
    // Extra line 3
    let b = literal!(222u32);
    let c = literal!(333u32);
}
"#;
    fs::write(&path, modified).unwrap();

    // Clear the file state cache so the runtime re-parses the file
    jeb_literal::clear_file_state_cache();

    // Find new positions
    let new_positions = find_all_literal_positions(&path);
    assert_eq!(new_positions.len(), 3, "Should still find 3 literals");

    let (new_a_line, new_a_col) = new_positions[0];
    let (new_b_line, new_b_col) = new_positions[1];
    let (new_c_line, new_c_col) = new_positions[2];

    println!("\n=== NEW POSITIONS ===");
    println!("A: line {} -> {} (unchanged)", a_line, new_a_line);
    println!("B: line {} -> {} (shifted)", b_line, new_b_line);
    println!("C: line {} -> {} (shifted)", c_line, new_c_line);

    // Access each literal at its new position
    let lit_a_after =
        jeb_literal::Literal::__new(10u32, path.to_str().unwrap(), new_a_line, new_a_col);
    let lit_b_after =
        jeb_literal::Literal::__new(20u32, path.to_str().unwrap(), new_b_line, new_b_col);
    let lit_c_after =
        jeb_literal::Literal::__new(30u32, path.to_str().unwrap(), new_c_line, new_c_col);

    // Verify each maintained its unique value
    assert_eq!(
        *lit_a_after, 111,
        "Literal A should maintain its value"
    );
    assert_eq!(
        *lit_b_after, 222,
        "Literal B should maintain its value despite line shift"
    );
    assert_eq!(
        *lit_c_after, 333,
        "Literal C should maintain its value despite line shift"
    );

    println!("\n✓ SUCCESS: All literals maintained distinct identities!");
    println!("  A: 111 (unchanged position)");
    println!("  B: 222 (shifted from line {} to {})", b_line, new_b_line);
    println!("  C: 333 (shifted from line {} to {})", c_line, new_c_line);

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_index_resolution_is_consistent() {
    // Test that the same literal always resolves to the same index,
    // even when accessed at different line numbers

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let original = r#"fn main() {
    let x = literal!(100u32);
}
"#;
    fs::write(&path, original).unwrap();

    env::set_var("LITERAL_MODE", "write");

    // Get initial position
    let positions = find_all_literal_positions(&path);
    let (line1, col1) = positions[0];

    println!("\n=== INITIAL ACCESS ===");
    println!("Line {}, column {}", line1, col1);

    // Resolve the index directly
    let index1 = jeb_literal::runtime::get_macro_index(&path, line1, col1).unwrap();
    println!("Resolved to index: {}", index1);

    // Modify the file to shift the literal
    let modified = r#"// New comment
// Another comment
fn main() {
    let x = literal!(100u32);
}
"#;
    fs::write(&path, modified).unwrap();

    // Clear the file state cache so the runtime re-parses the file
    jeb_literal::clear_file_state_cache();

    // Get new position
    let new_positions = find_all_literal_positions(&path);
    let (line2, col2) = new_positions[0];

    println!("\n=== AFTER LINE INSERTION ===");
    println!("Line {}, column {}", line2, col2);

    // Resolve the index again
    let index2 = jeb_literal::runtime::get_macro_index(&path, line2, col2).unwrap();
    println!("Resolved to index: {}", index2);

    // THE TEST: Both should resolve to the same index
    assert_eq!(
        index1, index2,
        "The same literal should always resolve to the same index, \
         regardless of line number. Got index {} at line {}, \
         but index {} at line {}.",
        index1, line1, index2, line2
    );

    println!("\n✓ SUCCESS: Index resolution is consistent!");
    println!("  Position changed: line {} -> {}", line1, line2);
    println!("  But index remained: {}", index1);

    env::remove_var("LITERAL_MODE");
}

/// Helper function to find all literal! macro positions in a file
fn find_all_literal_positions(path: &std::path::Path) -> Vec<(u32, u32)> {
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
