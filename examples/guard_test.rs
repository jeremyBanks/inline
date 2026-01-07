// Quick test of the guard-returning macro

use inline::cell;

fn main() {
    let mut counter = cell(0u32);

    println!("Counter value: {}", *counter);

    let current = *counter;
    counter.value = current + 1;

    println!("After increment: {}", *counter);

    // Test that we can't access the same value twice (would deadlock)
    // Uncomment to test:
    // let mut counter2 = cell(0u32);  // Would deadlock!
}
