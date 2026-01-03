use inline::inline;
use std::env;

#[test]
fn test_static_persistence_same_value() {
    env::set_var("INLINE_MODE", "memory");

    // Calling the macro from the same line should give the same underlying value
    fn get_value() -> inline::Inline<u32> {
        inline!(42u32)  // Always the same source location
    }

    let mut val1 = get_value();
    assert_eq!(*val1, 42);

    val1.set(100);
    drop(val1);  // Release lock

    // Get it again from same location - should see the updated value
    let val2 = get_value();
    assert_eq!(*val2, 100);

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_static_persistence_value_mutation() {
    env::set_var("INLINE_MODE", "memory");

    fn get_counter() -> inline::Inline<u32> {
        inline!(1u32)  // Always same source location
    }

    let mut val = get_counter();
    assert_eq!(*val, 1);

    val.set(100);
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
    let val1 = inline!(10u32);
    let val2 = inline!(20u32);
    let val3 = inline!(30u32);

    assert_eq!(*val1, 10);
    assert_eq!(*val2, 20);
    assert_eq!(*val3, 30);
}

#[test]
fn test_static_persistence_different_types() {
    // Same location but different types
    let val_u32 = inline!(42u32);
    let val_i32 = inline!(42i32);

    assert_eq!(*val_u32, 42u32);
    assert_eq!(*val_i32, 42i32);
}

#[test]
fn test_static_persistence_across_function_calls() {
    env::set_var("INLINE_MODE", "memory");

    fn increment_counter() -> u32 {
        let mut counter = inline!(0u32);
        let current = *counter;
        counter.set(current + 1);
        current + 1
    }

    // Call the function multiple times
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
    fn get_counter() -> inline::Inline<u32> {
        inline!(0u32)
    }

    // Spawn multiple threads that all access the same static value
    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..100 {
                    let mut counter = get_counter();
                    let current = *counter;
                    counter.set(current + 1);
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
