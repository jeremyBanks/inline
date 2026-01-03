// Example: Self-updating configuration
//
// This demonstrates using inline with more complex types via databake.
//
// To run:
// INLINE_MODE=write cargo run --example config

use inline::inline;

fn main() {
    // Simple configuration values that update themselves
    let max_retries = inline!(3u32);
    let timeout_ms = inline!(1000u32);
    let debug_mode = inline!(false);

    println!("Current configuration:");
    println!("  Max retries: {}", *max_retries.lock().get());
    println!("  Timeout: {}ms", *timeout_ms.lock().get());
    println!("  Debug mode: {}", *debug_mode.lock().get());

    // Simulate configuration changes
    println!("\nAdjusting configuration based on runtime conditions...");

    if *max_retries.lock().get() < 5 {
        println!("  Increasing max_retries to 5");
        max_retries.lock().set(5);
    }

    if *timeout_ms.lock().get() < 2000 {
        println!("  Increasing timeout to 2000ms");
        timeout_ms.lock().set(2000);
    }

    if !*debug_mode.lock().get() {
        println!("  Enabling debug mode");
        debug_mode.lock().set(true);
    }

    println!("\nConfiguration updated! Check the source file to see the changes.");
}
