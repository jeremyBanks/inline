use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn debug_litter_update() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.rs");

    // Write a simple file with qualified path
    let content = "fn main() {\n    let x = litter::litter!(42u32);\n}\n";
    fs::write(&path, content).unwrap();
    println!("Original file content:");
    println!("{}", content);

    // Find the position using the helper that looks for last segment
    let source = fs::read_to_string(&path).unwrap();
    let ast = syn::parse_file(&source).unwrap();

    use syn::visit::Visit;
    struct MacroFinder {
        line: Option<u32>,
        column: Option<u32>,
    }

    impl<'ast> Visit<'ast> for MacroFinder {
        fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
            // Check if this is a litter macro (might be just "litter" or "litter::litter")
            let is_litter = if let Some(segments) = node.mac.path.segments.iter().last() {
                segments.ident == "litter"
            } else {
                false
            };

            if is_litter {
                let span = node.mac.path.segments.last().unwrap().ident.span();
                let start = span.start();
                self.line = Some(start.line as u32);
                self.column = Some(start.column as u32);
                println!("Found litter macro at line {}, column {}", start.line, start.column);
            }
        }
    }

    let mut finder = MacroFinder { line: None, column: None };
    finder.visit_file(&ast);

    let line = finder.line.unwrap();
    let column = finder.column.unwrap();

    // Now try to update it
    env::set_var("LITTER_UPDATE", "1");

    let new_tokens: proc_macro2::TokenStream = "100u32".parse().unwrap();
    match litter::update_source_file(&path, line, column, new_tokens) {
        Ok(_) => println!("Update succeeded!"),
        Err(e) => println!("Update failed: {}", e),
    }

    println!("\nFile content after update:");
    println!("{}", fs::read_to_string(&path).unwrap());

    env::remove_var("LITTER_UPDATE");
}
