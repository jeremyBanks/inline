// Example: Self-updating configuration
//
// This demonstrates using inline with more complex types via databake.
//
// To run:
// LITERAL_MODE=write cargo run --example config

use code_cell::code_cell;

fn main() {
    // Simple configuration values that update themselves
    let mut max_retries = code_cell(3u32);
    let mut timeout_ms = code_cell(1000u32);
    let mut debug_mode = code_cell(false);

    println!("Current configuration:");
    println!("  Max retries: {}", *max_retries.get());
    println!("  Timeout: {}ms", *timeout_ms.get());
    println!("  Debug mode: {}", *debug_mode.get());

    // Simulate configuration changes
    println!("\nAdjusting configuration based on runtime conditions...");

    if *max_retries.get() < 5 {
        println!("  Increasing max_retries to 5");
        max_retries.value = 5;
    }

    if *timeout_ms.get() < 2000 {
        println!("  Increasing timeout to 2000ms");
        timeout_ms.value = 2000;
    }

    if !*debug_mode.get() {
        println!("  Enabling debug mode");
        debug_mode.value = true;
    }

    println!("\nConfiguration updated! Check the source file to see the changes.");
}
