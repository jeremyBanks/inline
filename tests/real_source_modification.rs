/// End-to-end tests that modify REAL source files
/// These tests actually modify the fixture source files on disk and verify the changes
///
/// CRITICAL: Each test modifies a DIFFERENT fixture file to allow parallel execution
mod fixtures;

use std::env;
use std::fs;
use std::panic;

#[test]
fn test_counter_a_modification() {
    env::set_var("LITTER_MODE", "write");

    let mut value = fixtures::counter_a::get();

    // Verify starts at default
    assert_eq!(*value.get(), 0u32);

    // Set to test value
    let test_val = 42u32;
    value.set(test_val);

    // Check disk actually changed
    let disk_content = fs::read_to_string("tests/fixtures/counter_a.rs").unwrap();
    assert!(
        disk_content.contains("litter!(42u32)"),
        "Disk should contain updated value. Content:\n{}",
        disk_content
    );

    // Restore to default
    value.set(0u32);

    // Check disk restored
    let disk_content = fs::read_to_string("tests/fixtures/counter_a.rs").unwrap();
    assert!(
        disk_content.contains("litter!(0u32)"),
        "Disk should be restored to default. Content:\n{}",
        disk_content
    );

    env::remove_var("LITTER_MODE");
}

#[test]
fn test_counter_b_modification() {
    env::set_var("LITTER_MODE", "write");

    let mut value = fixtures::counter_b::get();

    // Verify starts at default
    assert_eq!(*value.get(), 0u32);

    // Set to different test value
    let test_val = 999u32;
    value.set(test_val);

    // Check disk actually changed
    let disk_content = fs::read_to_string("tests/fixtures/counter_b.rs").unwrap();
    assert!(
        disk_content.contains("litter!(999u32)"),
        "Disk should contain updated value. Content:\n{}",
        disk_content
    );

    // Restore to default
    value.set(0u32);

    // Check disk restored
    let disk_content = fs::read_to_string("tests/fixtures/counter_b.rs").unwrap();
    assert!(
        disk_content.contains("litter!(0u32)"),
        "Disk should be restored to default. Content:\n{}",
        disk_content
    );

    env::remove_var("LITTER_MODE");
}

#[test]
fn test_counter_c_modification() {
    env::set_var("LITTER_MODE", "write");

    let mut value = fixtures::counter_c::get();

    // Verify starts at default
    assert_eq!(*value.get(), 0u32);

    // Set to yet another test value
    let test_val = 12345u32;
    value.set(test_val);

    // Check disk actually changed
    let disk_content = fs::read_to_string("tests/fixtures/counter_c.rs").unwrap();
    assert!(
        disk_content.contains("litter!(12345u32)"),
        "Disk should contain updated value. Content:\n{}",
        disk_content
    );

    // Restore to default
    value.set(0u32);

    // Check disk restored
    let disk_content = fs::read_to_string("tests/fixtures/counter_c.rs").unwrap();
    assert!(
        disk_content.contains("litter!(0u32)"),
        "Disk should be restored to default. Content:\n{}",
        disk_content
    );

    env::remove_var("LITTER_MODE");
}

#[test]
fn test_config_a_modification() {
    env::set_var("LITTER_MODE", "write");

    let mut value = fixtures::config_a::get();

    // Verify starts at default
    assert_eq!(value.get().as_str(), "default");

    // Set to test value
    let test_val = "test_config_value".to_string();
    value.set(test_val);

    // Check disk actually changed
    let disk_content = fs::read_to_string("tests/fixtures/config_a.rs").unwrap();
    assert!(
        disk_content.contains(r#""test_config_value""#),
        "Disk should contain updated value. Content:\n{}",
        disk_content
    );

    // Restore to default
    value.set("default".to_string());

    // Check disk restored
    let disk_content = fs::read_to_string("tests/fixtures/config_a.rs").unwrap();
    assert!(
        disk_content.contains(r#""default""#),
        "Disk should be restored to default. Content:\n{}",
        disk_content
    );

    env::remove_var("LITTER_MODE");
}

#[test]
fn test_multiple_modifications_same_value() {
    env::set_var("LITTER_MODE", "write");

    let mut value = fixtures::counter_d::get();

    // Do multiple modifications
    value.set(100u32);
    let disk = fs::read_to_string("tests/fixtures/counter_d.rs").unwrap();
    assert!(disk.contains("litter!(100u32)"));

    value.set(200u32);
    let disk = fs::read_to_string("tests/fixtures/counter_d.rs").unwrap();
    assert!(disk.contains("litter!(200u32)"));

    value.set(300u32);
    let disk = fs::read_to_string("tests/fixtures/counter_d.rs").unwrap();
    assert!(disk.contains("litter!(300u32)"));

    // Restore
    value.set(0u32);
    let disk = fs::read_to_string("tests/fixtures/counter_d.rs").unwrap();
    assert!(disk.contains("litter!(0u32)"));

    env::remove_var("LITTER_MODE");
}

#[test]
fn test_concurrent_modification_detection() {
    env::set_var("LITTER_MODE", "write");

    let mut value = fixtures::counter_e::get();

    // Verify starts at default
    assert_eq!(*value.get(), 0u32);

    // Force the file to be loaded by doing a set operation
    // This establishes the baseline for concurrent modification detection
    value.set(1u32);
    value.set(0u32); // Set back to default

    // NOW simulate external modification to the file
    // This mimics what would happen if another process modified the file
    // AFTER we've already loaded it
    fs::write(
        "tests/fixtures/counter_e.rs",
        r#"/// Fixture E: Counter for testing concurrent modification detection
/// Default value: 0
pub fn get() -> litter::Litter<u32> {
    litter::litter!(777u32)  // Externally modified!
}
"#,
    )
    .unwrap();

    // Now try to modify with our Litter value - this should panic!
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        value.set(100u32);
    }));

    // Verify we got the expected panic
    assert!(
        result.is_err(),
        "Expected panic due to concurrent modification detection"
    );

    // Check the panic message
    if let Err(panic_value) = result {
        let panic_msg = if let Some(s) = panic_value.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = panic_value.downcast_ref::<&str>() {
            s.to_string()
        } else {
            String::new()
        };

        assert!(
            panic_msg.contains("CONCURRENT MODIFICATION DETECTED"),
            "Expected panic message to mention concurrent modification, got: {}",
            panic_msg
        );
    }

    // Restore to default for cleanup
    fs::write(
        "tests/fixtures/counter_e.rs",
        r#"/// Fixture E: Counter for testing concurrent modification detection
/// Default value: 0
pub fn get() -> litter::Litter<u32> {
    litter::litter!(0u32)
}
"#,
    )
    .unwrap();

    env::remove_var("LITTER_MODE");
}

#[test]
fn test_formatting_preservation() {
    // This test demonstrates what happens to formatting when we modify a litter value
    env::set_var("LITTER_MODE", "write");

    // Capture the original content before any modifications
    let original_content = fs::read_to_string("tests/fixtures/counter_f.rs").unwrap();
    let original_lines: Vec<&str> = original_content.lines().collect();
    let original_line_count = original_lines.len();

    println!("=== ORIGINAL ===");
    println!("{}", original_content);
    println!("Line count: {}", original_line_count);

    // Now modify the value
    let mut value = fixtures::counter_f::get();
    value.set(42u32);

    // Read back the file and see what happened to formatting
    let modified_content = fs::read_to_string("tests/fixtures/counter_f.rs").unwrap();
    let modified_lines: Vec<&str> = modified_content.lines().collect();
    let modified_line_count = modified_lines.len();

    println!("=== MODIFIED ===");
    println!("{}", modified_content);
    println!("Line count: {}", modified_line_count);

    // Restore to default
    value.set(0u32);
    let restored_content = fs::read_to_string("tests/fixtures/counter_f.rs").unwrap();

    env::remove_var("LITTER_MODE");

    // Report findings
    println!("=== FORMATTING ANALYSIS ===");
    println!(
        "Original == Modified: {}",
        original_content == modified_content
    );
    println!(
        "Original == Restored: {}",
        original_content == restored_content
    );
    println!(
        "Line count changed: {} -> {}",
        original_line_count, modified_line_count
    );
}
