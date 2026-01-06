use jeb_literal::{literal, LiteralPrivate};
use std::env;

#[test]
fn test_static_persistence_same_value() {
    env::set_var("LITERAL_MODE", "memory");

    // Calling the macro from the same line should give the same underlying value
    fn get_value() -> jeb_literal::Literal<u32> {
        literal!(42u32)  // Always the same source location
    }

    let mut val1 = get_value();
    assert_eq!(*val1, 42);

    val1.literal = 100;
    drop(val1);  // Release lock

    // Get it again from same location - should see the updated value
    let val2 = get_value();
    assert_eq!(*val2, 100);

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_static_persistence_value_mutation() {
    env::set_var("LITERAL_MODE", "memory");

    fn get_counter() -> jeb_literal::Literal<u32> {
        literal!(1u32)  // Always same source location
    }

    let mut val = get_counter();
    assert_eq!(*val, 1);

    val.literal = 100;
    assert_eq!(*val, 100);

    drop(val);  // Release lock

    // Get again from same location - should persist
    let val2 = get_counter();
    assert_eq!(*val2, 100);

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_static_persistence_different_locations() {
    // Different locations should get different values
    let val1 = literal!(10u32);
    let val2 = literal!(20u32);
    let val3 = literal!(30u32);

    assert_eq!(*val1, 10);
    assert_eq!(*val2, 20);
    assert_eq!(*val3, 30);
}

#[test]
fn test_static_persistence_different_types() {
    // Same location but different types
    let val_u32 = literal!(42u32);
    let val_i32 = literal!(42i32);

    assert_eq!(*val_u32, 42u32);
    assert_eq!(*val_i32, 42i32);
}

#[test]
fn test_static_persistence_across_function_calls() {
    env::set_var("LITERAL_MODE", "memory");

    fn increment_counter() -> u32 {
        let mut counter = literal!(0u32);
        let current = *counter;
        counter.literal = current + 1;
        current + 1
    }

    // Call the function multiple times
    assert_eq!(increment_counter(), 1);
    assert_eq!(increment_counter(), 2);
    assert_eq!(increment_counter(), 3);
    assert_eq!(increment_counter(), 4);

    env::remove_var("LITERAL_MODE");
}

#[test]
fn test_static_persistence_thread_safety() {
    use std::thread;
    use std::sync::Arc;

    env::set_var("LITERAL_MODE", "memory");

    // Use a shared path that all threads can use
    // We use __new with a fixed path to avoid writing to the test source file
    let path = "/nonexistent/thread_test.rs";
    let path_arc = Arc::new(path.to_string());

    // Spawn multiple threads that all access the same registry entry
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let path = Arc::clone(&path_arc);
            thread::spawn(move || {
                for _ in 0..100 {
                    // All threads use the same (file, line, column) so they share state
                    let mut counter = jeb_literal::Literal::__new(0u32, &path, 1, 1);
                    let current = *counter;
                    counter.literal = current + 1;
                    // Lock is dropped here
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Check final value from same registry entry
    let counter = jeb_literal::Literal::__new(0u32, path, 1, 1);
    let final_value = *counter;

    // Should be 10 threads * 100 increments = 1000
    assert_eq!(final_value, 1000, "All increments should be accounted for");

    env::remove_var("LITERAL_MODE");
}
