use std::env;
use code_cell::CodeCellPrivate;

#[test]
fn test_lazy_loading_allows_missing_files_on_read() {
    // Creating a Inline for a non-existent file should not fail
    let value = code_cell::CodeCell::__new(42u32, "/nonexistent/path.rs", 1, 1);

    // Reading the value should work fine
    assert_eq!(*value.get(), 42u32);
    assert_eq!(*value, 42u32);

    println!("✓ Can create and read inline values even if source file doesn't exist");
}

#[test]
fn test_lazy_loading_write_fails_silently_for_missing_file() {
    // Ensure we're in a mode that requires file access
    env::set_var("CODE_CELL_MODE", "write");

    // Creating a Inline for a non-existent file should not fail
    {
        let mut value = code_cell::CodeCell::__new(42u32, "/nonexistent/path.rs", 1, 1);

        // Assigning to .value works in memory
        value.value = 100u32;
        assert_eq!(*value, 100u32);

        // Drop will attempt to write but silently fail (can't panic in Drop)
    }

    // Test passes - errors in Drop are silently handled

    env::remove_var("CODE_CELL_MODE");
}

#[test]
fn test_lazy_loading_memory_mode_works_without_file() {
    // In Memory mode, we should be able to set() without file access
    env::set_var("CODE_CELL_MODE", "memory");

    let mut value = code_cell::CodeCell::__new(42u32, "/nonexistent/path.rs", 1, 1);

    // This should work because we're in Memory mode
    value.value = 100u32;
    assert_eq!(*value, 100u32);

    env::remove_var("CODE_CELL_MODE");
}

#[test]
fn test_lazy_loading_verify_mode_fails_silently_for_missing_file() {
    // In Verify mode, we need file access
    env::set_var("CODE_CELL_MODE", "verify");

    // Use a different location than other tests to avoid registry collision
    {
        let mut value = code_cell::CodeCell::__new(42u32, "/nonexistent/verify_test.rs", 1, 1);

        // Assigning to .value works in memory
        // Use a different value so we don't hit the early return
        value.value = 100u32;
        assert_eq!(*value, 100u32);

        // Drop will attempt to verify but silently fail (can't panic in Drop)
    }

    // Test passes - errors in Drop are silently handled

    env::remove_var("CODE_CELL_MODE");
}
