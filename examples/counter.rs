// Example: Self-modifying counter
//
// This program increments a counter each time it runs, updating its own source code.
//
// To run:
// 1. Compile: rustc --edition 2021 -L ../target/debug/deps examples/counter.rs
// 2. Run with INLINE_MODE=write ./counter
// 3. Check the source file - the counter value will have been updated!

use inline::inline;

fn main() {
    let mut counter = inline!(0u32);

    println!("This program has been run {} times", *counter.get() + 1);

    let current = *counter.get();
    counter.set(current + 1);

    println!("\nThe counter has been updated in the source code!");
    println!("Run this program again to see it increment.");
}
