// Quick test of the guard-returning macro

use jeb_literal::literal;

fn main() {
    let mut counter = literal!(0u32);

    println!("Counter value: {}", *counter);

    let current = *counter;
    counter.set(current + 1);

    println!("After increment: {}", *counter);

    // Test that we can't access the same value twice (would deadlock)
    // Uncomment to test:
    // let mut counter2 = literal!(0u32);  // Would deadlock!
}
