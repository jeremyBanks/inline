/// End-to-end tests that modify REAL source files
/// These tests actually modify the fixture source files on disk and verify the changes
///
/// CRITICAL: Each test modifies a DIFFERENT fixture file to allow parallel execution
mod fixtures;

use std::env;
use std::fs;

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
