use std::env;

#[test]
fn test_lazy_loading_allows_missing_files_on_read() {
    // Creating a Inline for a non-existent file should not fail
    let value = jeb_literal::Literal::__new(42u32, "/nonexistent/path.rs", 1, 1);

    // Reading the value should work fine
    assert_eq!(*value.get(), 42u32);
    assert_eq!(*value, 42u32);

    println!("✓ Can create and read inline values even if source file doesn't exist");
}

#[test]
#[should_panic(expected = "Failed to access source file")]
fn test_lazy_loading_fails_on_set_for_missing_file() {
    // Ensure we're in a mode that requires file access
    env::set_var("LITERAL_MODE", "write");

    // Creating a Inline for a non-existent file should not fail
    let mut value = jeb_literal::Literal::__new(42u32, "/nonexistent/path.rs", 1, 1);

    // But trying to set() should fail because the file doesn't exist
    value.set(100u32);

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_lazy_loading_memory_mode_works_without_file() {
    // In Memory mode, we should be able to set() without file access
    env::set_var("LITERAL_MODE", "memory");

    let mut value = jeb_literal::Literal::__new(42u32, "/nonexistent/path.rs", 1, 1);

    // This should work because we're in Memory mode
    value.set(100u32);
    assert_eq!(*value, 100u32);

    env::remove_var("LITERAL_MODE");
}

#[test]
#[should_panic(expected = "Failed to access source file")]
fn test_lazy_loading_verify_mode_fails_for_missing_file() {
    // In Verify mode, we need file access
    env::set_var("LITERAL_MODE", "verify");

    // Use a different location than other tests to avoid registry collision
    let mut value = jeb_literal::Literal::__new(42u32, "/nonexistent/verify_test.rs", 1, 1);

    // This should fail because the file doesn't exist
    // Use a different value so we don't hit the early return
    value.set(100u32);

    env::remove_var("LITERAL_MODE");
}
