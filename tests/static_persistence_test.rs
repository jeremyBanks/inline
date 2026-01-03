use inline::registry::get_or_create;
use std::env;

#[test]
fn test_static_persistence_same_reference() {
    // Get two references from the same location
    let ref1 = get_or_create(42u32, file!(), 10, 20);
    let ref2 = get_or_create(99u32, file!(), 10, 20);

    // Should return the exact same reference (same address)
    assert_eq!(
        ref1 as *const _,
        ref2 as *const _,
        "Same location should return same reference"
    );

    // The initial value should be used (42, not 99)
    assert_eq!(*ref1.lock().get(), 42);
    assert_eq!(*ref2.lock().get(), 42);
}

#[test]
fn test_static_persistence_value_mutation() {
    env::set_var("INLINE_MODE", "memory");

    // Get a reference
    let ref1 = get_or_create(1u32, file!(), 30, 40);
    assert_eq!(*ref1.lock().get(), 1);

    // Modify it
    ref1.lock().set(100);

    // Get another reference to the "same location"
    let ref2 = get_or_create(1u32, file!(), 30, 40);

    // Should see the modified value
    assert_eq!(*ref2.lock().get(), 100);

    env::remove_var("INLINE_MODE");
}

#[test]
fn test_static_persistence_different_locations() {
    // Different locations should get different values
    let ref1 = get_or_create(10u32, file!(), 50, 10);
    let ref2 = get_or_create(20u32, file!(), 50, 20); // Different column
    let ref3 = get_or_create(30u32, file!(), 60, 10); // Different line

    // Should be different references
    assert_ne!(ref1 as *const _, ref2 as *const _);
    assert_ne!(ref1 as *const _, ref3 as *const _);
    assert_ne!(ref2 as *const _, ref3 as *const _);

    // Should have different values
    assert_eq!(*ref1.lock().get(), 10);
    assert_eq!(*ref2.lock().get(), 20);
    assert_eq!(*ref3.lock().get(), 30);
}

#[test]
fn test_static_persistence_different_types() {
    // Same location but different types should be different entries
    let ref_u32 = get_or_create(42u32, file!(), 70, 10);
    let ref_i32 = get_or_create(42i32, file!(), 70, 10);

    // These are different types, so different registry entries
    // We can't directly compare them since they're different types,
    // but we can verify they both work independently
    assert_eq!(*ref_u32.lock().get(), 42u32);
    assert_eq!(*ref_i32.lock().get(), 42i32);
}

#[test]
fn test_static_persistence_across_function_calls() {
    env::set_var("INLINE_MODE", "memory");

    fn increment_counter() -> u32 {
        let counter = get_or_create(0u32, file!(), 90, 23);
        let current = *counter.lock().get();
        counter.lock().set(current + 1);
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

    // Spawn multiple threads that all access the same static value
    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let counter = get_or_create(0u32, file!(), 110, 35);
                for _ in 0..100 {
                    let current = *counter.lock().get();
                    counter.lock().set(current + 1);
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Check final value
    let counter = get_or_create(0u32, file!(), 110, 35);
    let final_value = *counter.lock().get();

    // Should be 10 threads * 100 increments = 1000
    assert_eq!(final_value, 1000, "All increments should be accounted for");

    env::remove_var("INLINE_MODE");
}
