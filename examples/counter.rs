// Example: Self-modifying counter
//
// This program increments a counter each time it runs, updating its own source code.
//
// To run:
// 1. Compile: rustc --edition 2021 -L ../target/debug/deps examples/counter.rs
// 2. Run with LITTER_UPDATE=1 ./counter
// 3. Check the source file - the counter value will have been updated!

use litter::litter;

fn main() {
    let mut counter = litter!(0u32);

    println!("This program has been run {} times", *counter.get() + 1);

    counter.set(counter.get() + 1);

    println!("\nThe counter has been updated in the source code!");
    println!("Run this program again to see it increment.");
}
