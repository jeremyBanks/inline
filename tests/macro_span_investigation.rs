//! Investigation: How do macro spans interact with #[track_caller]?
//!
//! This test file explores whether #[track_caller] reports the macro call site
//! or the macro definition site when a macro expands to a function call.
//!
//! Also tests mixed macro/function usage in the same file with various orderings.

use std::panic::Location;
use std::collections::HashMap;

/// A function that captures its caller location
#[track_caller]
fn where_am_i() -> &'static Location<'static> {
    Location::caller()
}

/// A macro that calls where_am_i
macro_rules! where_am_i_macro {
    () => {
        where_am_i()
    };
}

#[test]
fn test_track_caller_with_direct_call() {
    let loc = where_am_i(); // Direct function call
    println!("Direct call: {}:{}:{}", loc.file(), loc.line(), loc.column());

    // This should report the line of the `where_am_i()` call above
    assert!(loc.file().ends_with("macro_span_investigation.rs"));
    assert_eq!(loc.line(), 22); // Line of `let loc = where_am_i();`
}

#[test]
fn test_track_caller_with_macro_call() {
    let loc = where_am_i_macro!(); // Macro that expands to function call
    println!("Macro call: {}:{}:{}", loc.file(), loc.line(), loc.column());

    // KEY QUESTION: Does this report line 31 (macro invocation)
    // or line 17 (inside macro definition)?
    assert!(loc.file().ends_with("macro_span_investigation.rs"));

    // If macros preserve call-site spans, this should be line 31
    // If macros use definition-site spans, this would be line 17
    let is_call_site = loc.line() == 31;
    let is_definition_site = loc.line() == 17;

    println!("Is call-site span: {}", is_call_site);
    println!("Is definition-site span: {}", is_definition_site);

    // We expect call-site based on Rust's span hygiene rules
    assert!(is_call_site, "Expected call-site span, got line {}", loc.line());
}

#[test]
fn test_track_caller_with_nested_macro() {
    macro_rules! outer {
        ($inner:expr) => {
            $inner
        };
    }

    let loc = outer!(where_am_i_macro!());
    println!("Nested macro call: {}:{}:{}", loc.file(), loc.line(), loc.column());

    // What line does this report?
    println!("Nested macro reports line: {}", loc.line());
}

#[test]
fn test_track_caller_column_position() {
    // Test what column is reported
    let loc1 = where_am_i_macro!(); let loc2 = where_am_i_macro!();

    println!("First macro:  {}:{}:{}", loc1.file(), loc1.line(), loc1.column());
    println!("Second macro: {}:{}:{}", loc2.file(), loc2.line(), loc2.column());

    // They should have different columns since they're on the same line
    assert_ne!(loc1.column(), loc2.column(), "Same-line macros should have different columns");
}

/// Test with a macro that has a path prefix (like inline::cell!)
mod inline_style {
    use super::*;

    #[macro_export]
    macro_rules! cell_style {
        ($value:expr) => {
            where_am_i()
        };
    }

    #[test]
    fn test_exported_macro_span() {
        let loc = cell_style!(42);
        println!("Exported macro: {}:{}:{}", loc.file(), loc.line(), loc.column());
        println!("Line reported: {}", loc.line());

        // Should report the call site, not the definition
        assert!(loc.line() > 80, "Expected call-site (line ~88), got {}", loc.line());
    }
}

// =============================================================================
// Mixed macro/function usage - simulating real inline crate usage
// =============================================================================

/// Simulates what inline::cell() does
#[track_caller]
fn fake_cell(value: i32) -> (i32, &'static Location<'static>) {
    (value, Location::caller())
}

/// Simulates inline::cell!() macro
macro_rules! fake_cell_macro {
    ($value:expr) => {
        fake_cell($value)
    };
}

#[test]
fn test_mixed_macro_and_function_same_file() {
    // Interleaved macro and function calls - this is the real-world scenario
    let (v1, loc1) = fake_cell(10);           // Function call
    let (v2, loc2) = fake_cell_macro!(20);    // Macro call
    let (v3, loc3) = fake_cell(30);           // Function call
    let (v4, loc4) = fake_cell_macro!(40);    // Macro call

    println!("Function 1: line {} col {}", loc1.line(), loc1.column());
    println!("Macro 1:    line {} col {}", loc2.line(), loc2.column());
    println!("Function 2: line {} col {}", loc3.line(), loc3.column());
    println!("Macro 2:    line {} col {}", loc4.line(), loc4.column());

    // All should have unique (line, column) pairs
    let positions: Vec<_> = [loc1, loc2, loc3, loc4]
        .iter()
        .map(|l| (l.line(), l.column()))
        .collect();

    // Check all unique
    let mut seen = HashMap::new();
    for (i, pos) in positions.iter().enumerate() {
        if let Some(prev) = seen.insert(pos, i) {
            panic!("Position {:?} used by both call {} and {}", pos, prev, i);
        }
    }
    println!("All positions unique: {:?}", positions);

    // Values should be correct
    assert_eq!((v1, v2, v3, v4), (10, 20, 30, 40));
}

#[test]
fn test_weird_access_order() {
    // Access in non-linear order (simulating complex control flow)
    let mut results = Vec::new();

    // Define closures that capture location
    let get_a = || fake_cell(100);
    let get_b = || fake_cell_macro!(200);
    let get_c = || fake_cell(300);

    // Access in weird order: C, A, B, A again, C again
    results.push(("C1", get_c()));
    results.push(("A1", get_a()));
    results.push(("B1", get_b()));
    results.push(("A2", get_a()));  // Same closure, same location expected
    results.push(("C2", get_c()));  // Same closure, same location expected

    for (name, (val, loc)) in &results {
        println!("{}: value={}, line={}, col={}", name, val, loc.line(), loc.column());
    }

    // A1 and A2 should have same location (same closure)
    assert_eq!(
        (results[1].1.1.line(), results[1].1.1.column()),
        (results[3].1.1.line(), results[3].1.1.column()),
        "Same closure should report same location"
    );

    // C1 and C2 should have same location (same closure)
    assert_eq!(
        (results[0].1.1.line(), results[0].1.1.column()),
        (results[4].1.1.line(), results[4].1.1.column()),
        "Same closure should report same location"
    );
}

#[test]
fn test_loop_with_mixed_calls() {
    // In a loop, each iteration should get the same location (it's the same source position)
    let mut func_locs = Vec::new();
    let mut macro_locs = Vec::new();

    for i in 0..3 {
        let (_, loc_f) = fake_cell(i);
        let (_, loc_m) = fake_cell_macro!(i * 10);
        func_locs.push((loc_f.line(), loc_f.column()));
        macro_locs.push((loc_m.line(), loc_m.column()));
    }

    println!("Function locations in loop: {:?}", func_locs);
    println!("Macro locations in loop: {:?}", macro_locs);

    // All function calls should have same location (same source line)
    assert!(func_locs.windows(2).all(|w| w[0] == w[1]),
            "Function calls in loop should have same location");

    // All macro calls should have same location (same source line)
    assert!(macro_locs.windows(2).all(|w| w[0] == w[1]),
            "Macro calls in loop should have same location");

    // But function and macro should have DIFFERENT locations
    assert_ne!(func_locs[0], macro_locs[0],
               "Function and macro should have different locations");
}

#[test]
fn test_same_line_function_and_macro() {
    // Function and macro on the SAME line - must be distinguishable by column
    let (_, loc_f) = fake_cell(1); let (_, loc_m) = fake_cell_macro!(2);

    println!("Same line - func: col {}, macro: col {}", loc_f.column(), loc_m.column());

    assert_eq!(loc_f.line(), loc_m.line(), "Should be on same line");
    assert_ne!(loc_f.column(), loc_m.column(), "Should have different columns");
}

#[test]
fn test_macro_inside_function_call() {
    // Macro result passed to function
    fn outer_fn(inner: (i32, &'static Location<'static>)) -> &'static Location<'static> {
        inner.1
    }

    let loc = outer_fn(fake_cell_macro!(42));
    println!("Macro inside function call: line {} col {}", loc.line(), loc.column());

    // Should report the macro location, not outer_fn location
    // The line should be this test function, not outer_fn definition
    assert!(loc.line() > 200, "Should be in test function, not outer_fn");
}
