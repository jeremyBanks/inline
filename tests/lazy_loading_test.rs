use std::env;

#[test]
fn test_lazy_loading_allows_missing_files_on_read() {
    // Creating a Litter for a non-existent file should not fail
    let value = litter::Litter::__new(42u32, "/nonexistent/path.rs", 1, 1);

    // Reading the value should work fine
    assert_eq!(*value.get(), 42u32);
    assert_eq!(*value, 42u32);

    println!("✓ Can create and read litter values even if source file doesn't exist");
}

#[test]
#[should_panic(expected = "Failed to access source file")]
fn test_lazy_loading_fails_on_set_for_missing_file() {
    // Ensure we're in a mode that requires file access
    env::set_var("LITTER_UPDATE", "1");

    // Creating a Litter for a non-existent file should not fail
    let mut value = litter::Litter::__new(42u32, "/nonexistent/path.rs", 1, 1);

    // But trying to set() should fail because the file doesn't exist
    value.set(100u32);

    env::remove_var("LITTER_UPDATE");
}

#[test]
fn test_lazy_loading_inactive_mode_works_without_file() {
    // In Inactive mode (default), we should be able to set() without file access
    env::remove_var("LITTER_UPDATE");
    env::remove_var("LITTER_VERIFY");
    env::set_var("LITTER_INACTIVE", "1");

    let mut value = litter::Litter::__new(42u32, "/nonexistent/path.rs", 1, 1);

    // This should work because we're in Inactive mode
    value.set(100u32);
    assert_eq!(*value, 100u32);

    env::remove_var("LITTER_INACTIVE");
}

#[test]
#[should_panic(expected = "Failed to access source file")]
fn test_lazy_loading_verify_mode_fails_for_missing_file() {
    // In Verify mode, we need file access
    env::set_var("LITTER_VERIFY", "1");
    env::remove_var("LITTER_UPDATE");

    let mut value = litter::Litter::__new(42u32, "/nonexistent/path.rs", 1, 1);

    // This should fail because the file doesn't exist
    // Use a different value so we don't hit the early return
    value.set(100u32);

    env::remove_var("LITTER_VERIFY");
}
