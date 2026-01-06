use std::env;
use std::fs;
use tempfile::TempDir;
use code_cell::CodeCellPrivate;

#[test]
fn test_very_long_value_formatting() {
    // What happens if we update to a VERY long value that might cause
    // prettyplease to wrap it across multiple lines?

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let original = r#"fn main() {
    let a = code_cell(1u32);
    let b = code_cell(2u32);
}
"#;
    fs::write(&path, original).unwrap();
    println!("=== ORIGINAL ===");
    println!("{}", original);

    env::set_var("CODE_CELL_MODE", "write");

    let positions = find_all_positions(&path);
    let (a_line, a_col) = positions[0];
    let (b_line, _b_col) = positions[1];

    println!("Initial: a at line {}, b at line {}", a_line, b_line);

    // Create a very large number (will this cause line wrapping?)
    let mut litter_a = code_cell::CodeCell::__new(1u32, path.to_str().unwrap(), a_line, a_col);

    // Update to maximum u32 value
    litter_a.value = 4294967295u32;

    println!("\n=== AFTER UPDATING A TO MAX U32 ===");
    let content = fs::read_to_string(&path).unwrap();
    println!("{}", content);

    let positions_after = find_all_positions(&path);
    println!("\nPositions after:");
    println!(
        "a at line {}, b at line {}",
        positions_after[0].0, positions_after[1].0
    );

    // Does b's line change?
    if positions_after[1].0 != b_line {
        println!(
            "\n❌ B's LINE NUMBER CHANGED FROM {} TO {}",
            b_line, positions_after[1].0
        );
        panic!("Line numbers shifted!");
    } else {
        println!("\n✓ Line numbers stable even with large value");
    }

    env::remove_var("CODE_CELL_MODE");
}

#[test]
fn test_tuple_value_formatting() {
    // What about a tuple that might format across multiple lines?

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    let original = r#"fn main() {
    let a = code_cell((1u32, 2u32, 3u32));
    let b = code_cell(100u32);
}
"#;
    fs::write(&path, original).unwrap();
    println!("=== ORIGINAL ===");
    println!("{}", original);

    env::set_var("CODE_CELL_MODE", "write");

    let positions = find_all_positions(&path);
    let (a_line, a_col) = positions[0];
    let (b_line, _b_col) = positions[1];

    println!("Initial: a at line {}, b at line {}", a_line, b_line);

    let mut litter_a =
        code_cell::CodeCell::__new((1u32, 2u32, 3u32), path.to_str().unwrap(), a_line, a_col);

    // Update to different tuple
    litter_a.value = (999u32, 888u32, 777u32);

    println!("\n=== AFTER UPDATING TUPLE ===");
    let content = fs::read_to_string(&path).unwrap();
    println!("{}", content);

    let positions_after = find_all_positions(&path);
    println!("\nPositions after:");
    println!(
        "a at line {}, b at line {}",
        positions_after[0].0, positions_after[1].0
    );

    assert_eq!(
        positions_after[1].0, b_line,
        "B's line should not change when updating tuple above it"
    );

    println!("\n✓ Line numbers stable with tuple values");

    env::remove_var("CODE_CELL_MODE");
}

fn find_all_positions(path: &std::path::Path) -> Vec<(u32, u32)> {
    let source = fs::read_to_string(path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct MacroCollector {
        positions: Vec<(u32, u32)>,
    }

    impl<'ast> Visit<'ast> for MacroCollector {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if let syn::Expr::Call(call) = node {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(segment) = path.path.segments.last() {
                        if segment.ident == "code_cell" {
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

    let mut collector = MacroCollector {
        positions: Vec::new(),
    };
    collector.visit_file(&ast);
    collector.positions
}
