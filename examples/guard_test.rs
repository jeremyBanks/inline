// Quick test of the guard-returning macro

use inline::inline;

fn main() {
    let mut counter = inline!(0u32);

    println!("Counter value: {}", **counter);

    let current = **counter;
    counter.set(current + 1);

    println!("After increment: {}", **counter);

    // Test that we can't access the same value twice (would deadlock)
    // Uncomment to test:
    // let mut counter2 = inline!(0u32);  // Would deadlock!
}
