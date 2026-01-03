# Test Strategy for Inline

## Testing Approach

We need to test the core functionality: modifying source code in place while maintaining AST stability.

## Test Infrastructure

### Helper Functions

```rust
// tests/common/mod.rs
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestFile {
    _dir: TempDir,  // Keeps dir alive
    pub path: PathBuf,
}

impl TestFile {
    pub fn new(content: &str) -> Self {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.rs");
        fs::write(&path, content).unwrap();
        TestFile { _dir: dir, path }
    }

    pub fn read(&self) -> String {
        fs::read_to_string(&self.path).unwrap()
    }

    pub fn contains(&self, s: &str) -> bool {
        self.read().contains(s)
    }

    pub fn assert_contains(&self, s: &str) {
        assert!(
            self.contains(s),
            "Expected file to contain: {}\n\nActual content:\n{}",
            s,
            self.read()
        );
    }
}
```

## Unit Tests

### Test 1: AST Parsing Preserves Spans

**Purpose**: Verify that `syn::parse_file` gives us accurate line/column information.

```rust
#[test]
fn test_span_preservation() {
    let source = r#"fn main() {
    let x = inline!(42);
}"#;

    let ast = syn::parse_file(source).unwrap();

    // Find the macro and check its span
    use syn::visit::Visit;

    struct MacroFinder {
        found_at: Option<(usize, usize)>,
    }

    impl<'ast> Visit<'ast> for MacroFinder {
        fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
            if let Some(ident) = node.mac.path.get_ident() {
                if ident == "inline" {
                    let span = ident.span();
                    let start = span.start();
                    self.found_at = Some((start.line, start.column));
                }
            }
        }
    }

    let mut finder = MacroFinder { found_at: None };
    finder.visit_file(&ast);

    assert_eq!(finder.found_at, Some((2, 13)),
        "Macro should be at line 2, column 13");
}
```

**Expected**: Pass if spans are preserved correctly.
**If fails**: We need alternative location tracking strategy.

### Test 2: FileState Load and Parse

**Purpose**: Verify FileState can load and parse a file.

```rust
#[test]
fn test_file_state_load() {
    let test_file = TestFile::new(r#"
use inline::inline;

fn main() {
    let x = inline!(42);
}
"#);

    let state = inline::runtime::FileState::load(&test_file.path);
    assert!(state.is_ok(), "Should successfully load file");
}
```

### Test 3: Macro Replacement

**Purpose**: Test the core replacement logic.

```rust
#[test]
fn test_macro_replacement() {
    let test_file = TestFile::new(r#"
use inline::inline;

fn main() {
    let x = inline!(42);
}
"#);

    let mut state = inline::runtime::FileState::load(&test_file.path).unwrap();

    // Replace the value at line 5, column 13 (approximately)
    let new_tokens: proc_macro2::TokenStream = "100".parse().unwrap();

    let result = state.replace_macro_value(5, 13, new_tokens);
    assert!(result.is_ok(), "Should successfully replace macro");

    // Write back and verify
    state.write_to_disk(&test_file.path).unwrap();

    test_file.assert_contains("inline!(100)");
    assert!(!test_file.contains("inline!(42)"));
}
```

### Test 4: Single Inline Update

**Purpose**: End-to-end test of updating a Inline value.

```rust
#[test]
fn test_single_litter_update() {
    // This test is tricky because we need to actually construct a Inline
    // instance that points to a real file...

    // For now, test the logic without the macro:
    let test_file = TestFile::new(r#"
fn test() {
    let x = inline!(42u32);
}
"#);

    let mut inline = inline::Inline::__new(
        42u32,
        test_file.path.to_str().unwrap(),
        3,  // line
        13, // column
    );

    // Update the value
    inline.set(100u32);

    // Check file was updated
    test_file.assert_contains("inline!(100u32)");
}
```

### Test 5: Multiple Litters in Same File

**Purpose**: Ensure multiple litters don't interfere.

```rust
#[test]
fn test_multiple_litters() {
    let test_file = TestFile::new(r#"
fn test() {
    let a = inline!(1u32);
    let b = inline!(2u32);
    let c = inline!(3u32);
}
"#);

    let mut litter_a = inline::Inline::__new(1u32, test_file.path.to_str().unwrap(), 3, 13);
    let mut litter_b = inline::Inline::__new(2u32, test_file.path.to_str().unwrap(), 4, 13);
    let mut litter_c = inline::Inline::__new(3u32, test_file.path.to_str().unwrap(), 5, 13);

    // Update in various orders
    litter_b.set(20u32);
    litter_a.set(10u32);
    litter_c.set(30u32);

    // Verify all updates persisted
    let content = test_file.read();
    assert!(content.contains("inline!(10u32)"));
    assert!(content.contains("inline!(20u32)"));
    assert!(content.contains("inline!(30u32)"));
}
```

### Test 6: databake Integration

**Purpose**: Verify databake types work correctly.

```rust
#[test]
fn test_databake_types() {
    use databake::Bake;

    let test_file = TestFile::new(r#"
fn test() {
    let v = inline!(vec![1, 2, 3]);
}
"#);

    // Test that Vec bakes correctly
    let vec = vec![1u32, 2, 3];
    let env = databake::CrateEnv::default();
    let baked = vec.bake(&env);

    // Should produce something like: vec![1u32, 2u32, 3u32]
    let baked_str = baked.to_string();
    assert!(baked_str.contains("vec!") || baked_str.contains("Vec::"));
}
```

### Test 7: Concurrent Updates

**Purpose**: Ensure file locking prevents corruption.

```rust
#[test]
fn test_concurrent_updates() {
    use std::sync::Arc;
    use std::thread;

    let test_file = TestFile::new(r#"
fn test() {
    let a = inline!(0u32);
    let b = inline!(0u32);
}
"#);

    let path = Arc::new(test_file.path.clone());

    let handles: Vec<_> = (0..10).map(|i| {
        let path = path.clone();
        thread::spawn(move || {
            let mut inline = inline::Inline::__new(
                0u32,
                path.to_str().unwrap(),
                3 + (i % 2) as u32,  // Alternate between two lines
                13,
            );
            inline.set(i as u32);
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // File should be valid Rust after all updates
    let content = test_file.read();
    let ast = syn::parse_file(&content);
    assert!(ast.is_ok(), "File should still be valid Rust");
}
```

### Test 8: Error Handling

**Purpose**: Graceful handling of errors.

```rust
#[test]
fn test_invalid_file() {
    let mut inline = inline::Inline::__new(
        42u32,
        "/nonexistent/file.rs",
        1,
        1,
    );

    // Should not panic, just fail gracefully
    inline.set(100u32);
    // (Error will be logged to stderr)
}

#[test]
fn test_wrong_location() {
    let test_file = TestFile::new(r#"
fn test() {
    let x = inline!(42u32);
}
"#);

    let mut state = inline::runtime::FileState::load(&test_file.path).unwrap();

    // Try to replace at wrong location
    let new_tokens: proc_macro2::TokenStream = "100".parse().unwrap();
    let result = state.replace_macro_value(999, 999, new_tokens);

    assert!(result.is_err(), "Should fail with wrong location");
}
```

## Integration Tests

### Self-Modifying Script Test

Create `tests/integration/self_modify.rs`:

```rust
use std::process::Command;
use std::fs;

#[test]
fn test_self_modifying_script() {
    // Copy template to temp location
    let script_content = r#"
use inline::inline;

fn main() {
    let mut counter = inline!(0u32);
    println!("Run #{}", *counter.get() + 1);
    counter.set(counter.get() + 1);
}
"#;

    let dir = tempfile::TempDir::new().unwrap();
    let script_path = dir.path().join("counter.rs");
    fs::write(&script_path, script_content).unwrap();

    // Run it 3 times
    for expected_run in 1..=3 {
        let output = Command::new("rustc")
            .arg(&script_path)
            .arg("-o")
            .arg(dir.path().join("counter"))
            .output()
            .unwrap();

        assert!(output.status.success(), "Compilation failed");

        let output = Command::new(dir.path().join("counter"))
            .output()
            .unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains(&format!("Run #{}", expected_run)));
    }

    // Verify final state
    let final_content = fs::read_to_string(&script_path).unwrap();
    assert!(final_content.contains("inline!(3u32)"));
}
```

## Test Execution Plan

### Phase 1: Foundation
1. Run Test 1 (span preservation) first - this validates our core assumption
2. Run Test 2 (file state load) - validates parsing
3. Run Test 3 (macro replacement) - validates AST modification

### Phase 2: Core Functionality
4. Run Test 4 (single update) - end-to-end single value
5. Run Test 5 (multiple litters) - multiple values in same file
6. Run Test 6 (databake) - type serialization

### Phase 3: Robustness
7. Run Test 7 (concurrency) - thread safety
8. Run Test 8 (errors) - error handling

### Phase 4: Real World
9. Run integration test (self-modifying script)

## Debugging Strategy

### If Test 1 Fails (Spans)
- Check if proc_macro2 spans are preserved
- Try alternative: use byte offsets instead
- Or implement line/column counting manually

### If Test 3 Fails (Replacement)
- Print the AST structure
- Check if VisitMut is traversing correctly
- Verify target line/column calculations

### If Test 5 Fails (Multiple)
- Check FileState sharing logic
- Verify locking prevents corruption
- Check if file state is reloaded after updates

### If Test 7 Fails (Concurrency)
- Add debug logging to see lock acquisition
- Check for deadlocks
- Verify Arc usage is correct

## Success Metrics

- [ ] All unit tests pass
- [ ] Integration test passes
- [ ] No panics under concurrent load
- [ ] File remains valid Rust after updates
- [ ] Updates are durable (persist across processes)

## Performance Benchmarks (Optional)

```rust
#[bench]
fn bench_single_update(b: &mut Bencher) {
    let test_file = TestFile::new("...");
    let mut inline = inline::Inline::__new(...);

    b.iter(|| {
        inline.set(black_box(42u32));
    });
}
```

Expected: < 10ms per update (parse + format + write)
