// Example: Self-updating configuration
//
// This demonstrates using inline with more complex types via databake.
//
// To run:
// LITERAL_MODE=write cargo run --example config

use jeb_literal::literal;

fn main() {
    // Simple configuration values that update themselves
    let mut max_retries = literal!(3u32);
    let mut timeout_ms = literal!(1000u32);
    let mut debug_mode = literal!(false);

    println!("Current configuration:");
    println!("  Max retries: {}", *max_retries.get());
    println!("  Timeout: {}ms", *timeout_ms.get());
    println!("  Debug mode: {}", *debug_mode.get());

    // Simulate configuration changes
    println!("\nAdjusting configuration based on runtime conditions...");

    if *max_retries.get() < 5 {
        println!("  Increasing max_retries to 5");
        max_retries.set(5);
    }

    if *timeout_ms.get() < 2000 {
        println!("  Increasing timeout to 2000ms");
        timeout_ms.set(2000);
    }

    if !*debug_mode.get() {
        println!("  Enabling debug mode");
        debug_mode.set(true);
    }

    println!("\nConfiguration updated! Check the source file to see the changes.");
}
