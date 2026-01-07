#!/usr/bin/env rust
//! Cargo subcommand to regenerate inline cell snapshots.
//!
//! Usage:
//!   cargo inline-write [CARGO_TEST_ARGS...]
//!
//! This is equivalent to:
//!   INLINE_MODE=write cargo test [ARGS...]
//!
//! Examples:
//!   cargo inline-write
//!   cargo inline-write -- --nocapture
//!   cargo inline-write test_name

use std::env;
use std::process::{Command, exit};

fn main() {
    // Cargo invokes this as: cargo-inline-write inline-write [args...]
    // We need to skip the first argument if it's the subcommand name
    let mut args: Vec<String> = env::args().collect();

    // Remove the binary name
    args.remove(0);

    // If the first arg is "inline-write", remove it (cargo passes the subcommand name)
    if args.first().map(|s| s.as_str()) == Some("inline-write") {
        args.remove(0);
    }

    // Set the environment variable to enable write mode
    env::set_var("INLINE_MODE", "write");

    // Execute cargo test with all the provided arguments
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.args(&args);

    // Preserve the current environment (including our INLINE_MODE=write)
    cmd.envs(env::vars());

    // Execute and forward the exit code
    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("Failed to execute cargo test: {}", e);
        exit(1);
    });

    exit(status.code().unwrap_or(1));
}
