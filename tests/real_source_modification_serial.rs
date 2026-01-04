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
    env::set_var("LITERAL_MODE", "write");

    // Verify starts at default
    {
        let value = fixtures::counter_a::get();
        assert_eq!(*value.get(), 0u32);
    }

    // Set to test value
    {
        let mut value = fixtures::counter_a::get();
        let test_val = 42u32;
        value.literal = test_val;
        // Drop triggers write
    }

    // Check disk actually changed
    let disk_content = fs::read_to_string("tests/fixtures/counter_a.rs").unwrap();
    assert!(
        disk_content.contains("literal!(42u32)"),
        "Disk should contain updated value. Content:\n{}",
        disk_content
    );

    // Restore to default
    {
        let mut value = fixtures::counter_a::get();
        value.literal = 0u32;
        // Drop triggers write
    }

    // Check disk restored
    let disk_content = fs::read_to_string("tests/fixtures/counter_a.rs").unwrap();
    assert!(
        disk_content.contains("literal!(0u32)"),
        "Disk should be restored to default. Content:\n{}",
        disk_content
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_counter_b_modification() {
    env::set_var("LITERAL_MODE", "write");

    // Verify starts at default
    {
        let value = fixtures::counter_b::get();
        assert_eq!(*value.get(), 0u32);
    }

    // Set to different test value
    {
        let mut value = fixtures::counter_b::get();
        let test_val = 999u32;
        value.literal = test_val;
        // Drop triggers write
    }

    // Check disk actually changed
    let disk_content = fs::read_to_string("tests/fixtures/counter_b.rs").unwrap();
    assert!(
        disk_content.contains("literal!(999u32)"),
        "Disk should contain updated value. Content:\n{}",
        disk_content
    );

    // Restore to default
    {
        let mut value = fixtures::counter_b::get();
        value.literal = 0u32;
        // Drop triggers write
    }

    // Check disk restored
    let disk_content = fs::read_to_string("tests/fixtures/counter_b.rs").unwrap();
    assert!(
        disk_content.contains("literal!(0u32)"),
        "Disk should be restored to default. Content:\n{}",
        disk_content
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_counter_c_modification() {
    env::set_var("LITERAL_MODE", "write");

    // Verify starts at default
    {
        let value = fixtures::counter_c::get();
        assert_eq!(*value.get(), 0u32);
    }

    // Set to yet another test value
    {
        let mut value = fixtures::counter_c::get();
        let test_val = 12345u32;
        value.literal = test_val;
        // Drop triggers write
    }

    // Check disk actually changed
    let disk_content = fs::read_to_string("tests/fixtures/counter_c.rs").unwrap();
    assert!(
        disk_content.contains("literal!(12345u32)"),
        "Disk should contain updated value. Content:\n{}",
        disk_content
    );

    // Restore to default
    {
        let mut value = fixtures::counter_c::get();
        value.literal = 0u32;
        // Drop triggers write
    }

    // Check disk restored
    let disk_content = fs::read_to_string("tests/fixtures/counter_c.rs").unwrap();
    assert!(
        disk_content.contains("literal!(0u32)"),
        "Disk should be restored to default. Content:\n{}",
        disk_content
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_config_a_modification() {
    env::set_var("LITERAL_MODE", "write");

    // Verify starts at default
    {
        let value = fixtures::config_a::get();
        assert_eq!(value.get().as_str(), "default");
    }

    // Set to test value
    {
        let mut value = fixtures::config_a::get();
        let test_val = "test_config_value".to_string();
        value.literal = test_val;
        // Drop triggers write
    }

    // Check disk actually changed
    let disk_content = fs::read_to_string("tests/fixtures/config_a.rs").unwrap();
    assert!(
        disk_content.contains(r#""test_config_value""#),
        "Disk should contain updated value. Content:\n{}",
        disk_content
    );

    // Restore to default
    {
        let mut value = fixtures::config_a::get();
        value.literal = "default".to_string();
        // Drop triggers write
    }

    // Check disk restored
    let disk_content = fs::read_to_string("tests/fixtures/config_a.rs").unwrap();
    assert!(
        disk_content.contains(r#""default""#),
        "Disk should be restored to default. Content:\n{}",
        disk_content
    );

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_multiple_modifications_same_value() {
    env::set_var("LITERAL_MODE", "write");

    // Do multiple modifications
    {
        let mut value = fixtures::counter_d::get();
        value.literal = 100u32;
        // Drop triggers write
    }
    let disk = fs::read_to_string("tests/fixtures/counter_d.rs").unwrap();
    assert!(disk.contains("literal!(100u32)"));

    {
        let mut value = fixtures::counter_d::get();
        value.literal = 200u32;
        // Drop triggers write
    }
    let disk = fs::read_to_string("tests/fixtures/counter_d.rs").unwrap();
    assert!(disk.contains("literal!(200u32)"));

    {
        let mut value = fixtures::counter_d::get();
        value.literal = 300u32;
        // Drop triggers write
    }
    let disk = fs::read_to_string("tests/fixtures/counter_d.rs").unwrap();
    assert!(disk.contains("literal!(300u32)"));

    // Restore
    {
        let mut value = fixtures::counter_d::get();
        value.literal = 0u32;
        // Drop triggers write
    }
    let disk = fs::read_to_string("tests/fixtures/counter_d.rs").unwrap();
    assert!(disk.contains("literal!(0u32)"));

    env::remove_var("LITERAL_MODE");
}

#[test]
#[ignore] // TODO: Adapt for write-on-drop - concurrent modification detection needs redesign
fn test_concurrent_modification_detection() {
    env::set_var("LITERAL_MODE", "write");

    let mut value = fixtures::counter_e::get();

    // Verify starts at default
    assert_eq!(*value, 0u32);

    // Force the file to be loaded by doing a set operation
    // This establishes the baseline for concurrent modification detection
    value.literal = 1u32;
    value.literal = 0u32; // Set back to default

    // NOW simulate external modification to the file
    // This mimics what would happen if another process modified the file
    // AFTER we've already loaded it
    fs::write(
        "tests/fixtures/counter_e.rs",
        r#"/// Fixture E: Counter for testing concurrent modification detection
/// Default value: 0
pub fn get() -> jeb_literal::Literal<u32> {
    jeb_literal::literal!(777u32)  // Externally modified!
}
"#,
    )
    .unwrap();

    // Now try to modify with our Inline value - this should panic!
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        value.literal = 100u32;
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
pub fn get() -> jeb_literal::Literal<u32> {
    jeb_literal::literal!(0u32)
}
"#,
    )
    .unwrap();

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_formatting_preservation() {
    // This test demonstrates what happens to formatting when we modify a inline value
    env::set_var("LITERAL_MODE", "write");

    // Capture the original content before any modifications
    let original_content = fs::read_to_string("tests/fixtures/counter_f.rs").unwrap();
    let original_lines: Vec<&str> = original_content.lines().collect();
    let original_line_count = original_lines.len();

    println!("=== ORIGINAL ===");
    println!("{}", original_content);
    println!("Line count: {}", original_line_count);

    // Now modify the value
    let mut value = fixtures::counter_f::get();
    value.literal = 42u32;

    // Read back the file and see what happened to formatting
    let modified_content = fs::read_to_string("tests/fixtures/counter_f.rs").unwrap();
    let modified_lines: Vec<&str> = modified_content.lines().collect();
    let modified_line_count = modified_lines.len();

    println!("=== MODIFIED ===");
    println!("{}", modified_content);
    println!("Line count: {}", modified_line_count);

    // Restore to default
    value.literal = 0u32;
    let restored_content = fs::read_to_string("tests/fixtures/counter_f.rs").unwrap();

    env::remove_var("LITERAL_MODE");

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

#[test]
#[ignore] // TODO: Adapt for write-on-drop - needs explicit drops between modifications
fn test_multiple_macros_same_file() {
    // Test modifying multiple different literal! macros in the same file
    // in various orders, including on the same line and different lines
    env::set_var("LITERAL_MODE", "write");

    let mut first = fixtures::counter_g::get_first();
    let mut second = fixtures::counter_g::get_second();
    let mut third = fixtures::counter_g::get_third();

    // Verify initial values
    assert_eq!(*first.get(), 10u32);
    assert_eq!(*second.get(), 20u32);
    assert_eq!(*third.get(), 30u32);

    // Modify first macro
    first.literal = 100u32;
    let disk = fs::read_to_string("tests/fixtures/counter_g.rs").unwrap();
    assert!(disk.contains("literal!(100u32)"));
    assert!(disk.contains("literal!(20u32)"));
    assert!(disk.contains("literal!(30u32)"));

    // Modify second macro
    second.literal = 200u32;
    let disk = fs::read_to_string("tests/fixtures/counter_g.rs").unwrap();
    assert!(disk.contains("literal!(100u32)"));
    assert!(disk.contains("literal!(200u32)"));
    assert!(disk.contains("literal!(30u32)"));

    // Modify third macro
    third.literal = 300u32;
    let disk = fs::read_to_string("tests/fixtures/counter_g.rs").unwrap();
    assert!(disk.contains("literal!(100u32)"));
    assert!(disk.contains("literal!(200u32)"));
    assert!(disk.contains("literal!(300u32)"));

    // Modify first again
    first.literal = 111u32;
    let disk = fs::read_to_string("tests/fixtures/counter_g.rs").unwrap();
    assert!(disk.contains("literal!(111u32)"));
    assert!(disk.contains("literal!(200u32)"));
    assert!(disk.contains("literal!(300u32)"));

    // Modify in reverse order
    third.literal = 333u32;
    second.literal = 222u32;
    first.literal = 11u32;
    let disk = fs::read_to_string("tests/fixtures/counter_g.rs").unwrap();
    assert!(disk.contains("literal!(11u32)"));
    assert!(disk.contains("literal!(222u32)"));
    assert!(disk.contains("literal!(333u32)"));

    // Restore all to defaults
    first.literal = 10u32;
    second.literal = 20u32;
    third.literal = 30u32;
    let disk = fs::read_to_string("tests/fixtures/counter_g.rs").unwrap();
    assert!(disk.contains("literal!(10u32)"));
    assert!(disk.contains("literal!(20u32)"));
    assert!(disk.contains("literal!(30u32)"));

    env::remove_var("LITERAL_MODE");
}

#[test]
#[ignore] // TODO: Adapt for write-on-drop - needs explicit drops between modifications
fn test_multiple_files_interleaved() {
    // Test modifying macros across multiple files in arbitrary order
    env::set_var("LITERAL_MODE", "write");

    let mut counter_a = fixtures::counter_a::get();
    let mut counter_b = fixtures::counter_b::get();
    let mut config_a = fixtures::config_a::get();
    let mut counter_d = fixtures::counter_d::get();

    // Verify all start at defaults
    assert_eq!(*counter_a.get(), 0u32);
    assert_eq!(*counter_b.get(), 0u32);
    assert_eq!(config_a.get().as_str(), "default");
    assert_eq!(*counter_d.get(), 0u32);

    // Modify in arbitrary interleaved order
    counter_a.literal = 1u32;
    assert!(fs::read_to_string("tests/fixtures/counter_a.rs")
        .unwrap()
        .contains("literal!(1u32)"));

    counter_b.literal = 2u32;
    assert!(fs::read_to_string("tests/fixtures/counter_b.rs")
        .unwrap()
        .contains("literal!(2u32)"));

    counter_a.literal = 11u32; // Modify counter_a again
    assert!(fs::read_to_string("tests/fixtures/counter_a.rs")
        .unwrap()
        .contains("literal!(11u32)"));

    config_a.literal = "test_value".to_string();
    assert!(fs::read_to_string("tests/fixtures/config_a.rs")
        .unwrap()
        .contains(r#""test_value""#));

    counter_d.literal = 4u32;
    assert!(fs::read_to_string("tests/fixtures/counter_d.rs")
        .unwrap()
        .contains("literal!(4u32)"));

    counter_b.literal = 22u32; // Modify counter_b again
    assert!(fs::read_to_string("tests/fixtures/counter_b.rs")
        .unwrap()
        .contains("literal!(22u32)"));

    counter_a.literal = 111u32; // Modify counter_a third time
    assert!(fs::read_to_string("tests/fixtures/counter_a.rs")
        .unwrap()
        .contains("literal!(111u32)"));

    // Verify all files have correct values
    assert!(fs::read_to_string("tests/fixtures/counter_a.rs")
        .unwrap()
        .contains("literal!(111u32)"));
    assert!(fs::read_to_string("tests/fixtures/counter_b.rs")
        .unwrap()
        .contains("literal!(22u32)"));
    assert!(fs::read_to_string("tests/fixtures/config_a.rs")
        .unwrap()
        .contains(r#""test_value""#));
    assert!(fs::read_to_string("tests/fixtures/counter_d.rs")
        .unwrap()
        .contains("literal!(4u32)"));

    // Restore all to defaults
    counter_a.literal = 0u32;
    counter_b.literal = 0u32;
    config_a.literal = "default".to_string();
    counter_d.literal = 0u32;

    // Verify restoration
    assert!(fs::read_to_string("tests/fixtures/counter_a.rs")
        .unwrap()
        .contains("literal!(0u32)"));
    assert!(fs::read_to_string("tests/fixtures/counter_b.rs")
        .unwrap()
        .contains("literal!(0u32)"));
    assert!(fs::read_to_string("tests/fixtures/config_a.rs")
        .unwrap()
        .contains(r#""default""#));
    assert!(fs::read_to_string("tests/fixtures/counter_d.rs")
        .unwrap()
        .contains("literal!(0u32)"));

    env::remove_var("LITERAL_MODE");
}
