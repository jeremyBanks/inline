use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn debug_litter_update() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Write a simple file with qualified path
    let content = "fn main() {\n    let x = inline::code_cell(42u32);\n}\n";
    fs::write(&path, content).unwrap();
    println!("Original file content:");
    println!("{}", content);

    // Find the position using the helper that looks for last segment
    let source = fs::read_to_string(&path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct LiteralFinder {
        line: Option<u32>,
        column: Option<u32>,
    }

    impl<'ast> Visit<'ast> for LiteralFinder {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            // Check if this is a call to code_cell()
            if let syn::Expr::Call(call) = node {
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(segment) = path.path.segments.last() {
                        if segment.ident == "code_cell" {
                            let span = segment.ident.span();
                            let start = span.start();
                            self.line = Some(start.line as u32);
                            self.column = Some(start.column as u32);
                            println!(
                                "Found code_cell() call at line {}, column {}",
                                start.line, start.column
                            );
                        }
                    }
                }
            }
            syn::visit::visit_expr(self, node);
        }
    }

    let mut finder = LiteralFinder {
        line: None,
        column: None,
    };
    finder.visit_file(&ast);

    let line = finder.line.unwrap();
    let column = finder.column.unwrap();

    // Now try to update it
    env::set_var("CODE_CELL_MODE", "write");

    let new_tokens: proc_macro2::TokenStream = "100u32".parse().unwrap();
    match code_cell::update_source_file(&path, line, column, new_tokens) {
        Ok(_) => println!("Update succeeded!"),
        Err(e) => println!("Update failed: {}", e),
    }

    println!("\nFile content after update:");
    println!("{}", fs::read_to_string(&path).unwrap());

    env::remove_var("CODE_CELL_MODE");
}
