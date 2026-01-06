use std::env;
use std::fs;
use tempfile::TempDir;
use code_cell::CodeCellPrivate;

#[test]
fn test_multiple_updates_same_litter() {
    // This tests the scenario: compile once, update multiple times
    // The concern: does the position stay stable across multiple updates?

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let original = r#"fn main() {
    let x = code_cell(42u32);
}
"#;
    fs::write(&path, original).unwrap();

    env::set_var("CODE_CELL_MODE", "write");

    // Find initial position
    let source = fs::read_to_string(&path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct MacroFinder {
        line: Option<u32>,
        column: Option<u32>,
    }

    impl<'ast> Visit<'ast> for MacroFinder {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if let syn::Expr::Call(call) = node {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(segment) = path.path.segments.last() {
                        if segment.ident == "code_cell" {
                            let span = segment.ident.span();
                            let start = span.start();
                            self.line = Some(start.line as u32);
                            self.column = Some(start.column as u32);
                        }
                    }
                }
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut finder = MacroFinder {
        line: None,
        column: None,
    };
    finder.visit_file(&ast);
    let (original_line, original_column) = (finder.line.unwrap(), finder.column.unwrap());

    println!(
        "Original position: line {}, column {}",
        original_line, original_column
    );

    // Create a Inline instance with the captured position
    let mut value = code_cell::CodeCell::__new(
        42u32,
        path.to_str().unwrap(),
        original_line,
        original_column,
    );

    // Update 1: 42 -> 100
    value.value = 100u32;
    println!("After update 1:");
    println!("{}", fs::read_to_string(&path).unwrap());

    // Parse again and check if position is still the same
    let source = fs::read_to_string(&path).unwrap();
    let ast = syn::parse_file(&source).unwrap();
    let mut finder = MacroFinder {
        line: None,
        column: None,
    };
    finder.visit_file(&ast);
    let (line_after_1, col_after_1) = (finder.line.unwrap(), finder.column.unwrap());

    assert_eq!(
        line_after_1, original_line,
        "Line should not change after update 1"
    );
    assert_eq!(
        col_after_1, original_column,
        "Column should not change after update 1"
    );

    // Update 2: 100 -> 999
    value.value = 999u32;
    println!("After update 2:");
    println!("{}", fs::read_to_string(&path).unwrap());

    // Check position again
    let source = fs::read_to_string(&path).unwrap();
    let ast = syn::parse_file(&source).unwrap();
    let mut finder = MacroFinder {
        line: None,
        column: None,
    };
    finder.visit_file(&ast);
    let (line_after_2, col_after_2) = (finder.line.unwrap(), finder.column.unwrap());

    assert_eq!(
        line_after_2, original_line,
        "Line should not change after update 2"
    );
    assert_eq!(
        col_after_2, original_column,
        "Column should not change after update 2"
    );

    // Update 3: 999 -> 1
    value.value = 1u32;
    println!("After update 3:");
    println!("{}", fs::read_to_string(&path).unwrap());

    // Final position check
    let source = fs::read_to_string(&path).unwrap();
    let ast = syn::parse_file(&source).unwrap();
    let mut finder = MacroFinder {
        line: None,
        column: None,
    };
    finder.visit_file(&ast);
    let (line_after_3, col_after_3) = (finder.line.unwrap(), finder.column.unwrap());

    assert_eq!(
        line_after_3, original_line,
        "Line should not change after update 3"
    );
    assert_eq!(
        col_after_3, original_column,
        "Column should not change after update 3"
    );

    println!("\n✓ Position remained stable through 3 updates!");
    println!(
        "  Original: line {}, column {}",
        original_line, original_column
    );
    println!("  Final:    line {}, column {}", line_after_3, col_after_3);

    env::remove_var("CODE_CELL_MODE");
}
