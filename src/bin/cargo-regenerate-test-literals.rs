#!/usr/bin/env rust
//! Cargo subcommand to regenerate literal test snapshots.
//!
//! Usage:
//!   cargo regenerate-test-literals [CARGO_TEST_ARGS...]
//!
//! This is equivalent to:
//!   LITERAL_MODE=write cargo test [ARGS...]
//!
//! Examples:
//!   cargo regenerate-test-literals
//!   cargo regenerate-test-literals -- --nocapture
//!   cargo regenerate-test-literals test_name

use std::env;
use std::process::{Command, exit};

fn main() {
    // Cargo invokes this as: cargo-regenerate-test-literals regenerate-test-literals [args...]
    // We need to skip the first argument if it's the subcommand name
    let mut args: Vec<String> = env::args().collect();

    // Remove the binary name
    args.remove(0);

    // If the first arg is "regenerate-test-literals", remove it (cargo passes the subcommand name)
    if args.first().map(|s| s.as_str()) == Some("regenerate-test-literals") {
        args.remove(0);
    }

    // Set the environment variable to enable write mode
    env::set_var("LITERAL_MODE", "write");

    // Execute cargo test with all the provided arguments
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.args(&args);

    // Preserve the current environment (including our LITERAL_MODE=write)
    cmd.envs(env::vars());

    // Execute and forward the exit code
    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("Failed to execute cargo test: {}", e);
        exit(1);
    });

    exit(status.code().unwrap_or(1));
}
