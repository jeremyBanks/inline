use inline::cell;
use std::env;

#[test]
fn test_static_persistence_same_value() {
    env::set_var("INLINE_MODE", "memory");

    // Calling the macro from the same line should give the same underlying value
    fn get_value() -> inline::InlineCell<u32> {
        cell(42u32)  // Always the same source location
    }

    let mut val1 = get_value();
    assert_eq!(*val1, 42);

    val1.value = 100;
    drop(val1);  // Release lock

    // Get it again from same location - should see the updated value
    let val2 = get_value();
    assert_eq!(*val2, 100);

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_static_persistence_value_mutation() {
    env::set_var("INLINE_MODE", "memory");

    fn get_counter() -> inline::InlineCell<u32> {
        cell(1u32)  // Always same source location
    }

    let mut val = get_counter();
    assert_eq!(*val, 1);

    val.value = 100;
    assert_eq!(*val, 100);

    drop(val);  // Release lock

    // Get again from same location - should persist
    let val2 = get_counter();
    assert_eq!(*val2, 100);

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_static_persistence_different_locations() {
    // Different locations should get different values
    let val1 = cell(10u32);
    let val2 = cell(20u32);
    let val3 = cell(30u32);

    assert_eq!(*val1, 10);
    assert_eq!(*val2, 20);
    assert_eq!(*val3, 30);
}

#[test]
fn test_static_persistence_different_types() {
    // Same location but different types
    let val_u32 = cell(42u32);
    let val_i32 = cell(42i32);

    assert_eq!(*val_u32, 42u32);
    assert_eq!(*val_i32, 42i32);
}

#[test]
fn test_static_persistence_across_function_calls() {
    env::set_var("INLINE_MODE", "memory");

    fn increment_counter() -> u32 {
        let mut counter = cell(1u32);
        let current = *counter;
        counter.value = current + 1;
        current  // Return the value before incrementing
    }

    // Call the function multiple times - each returns the current value, then increments
    assert_eq!(increment_counter(), 1);
    assert_eq!(increment_counter(), 2);
    assert_eq!(increment_counter(), 3);
    assert_eq!(increment_counter(), 4);

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_static_persistence_thread_safety() {
    use std::thread;

    env::set_var("INLINE_MODE", "memory");

    // Helper function to ensure all threads access the same source location
    fn get_counter() -> inline::InlineCell<u32> {
        cell(0u32)
    }

    // Spawn multiple threads that all access the same static value
    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..100 {
                    let mut counter = get_counter();
                    let current = *counter;
                    counter.value = current + 1;
                    // Lock is dropped here
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Check final value from same location
    let counter = get_counter();
    let final_value = *counter;

    // Should be 10 threads * 100 increments = 1000
    assert_eq!(final_value, 1000, "All increments should be accounted for");

    env::remove_var("INLINE_MODE");
}
