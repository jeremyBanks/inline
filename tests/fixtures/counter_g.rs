/// Fixture G: Multiple counters in the same file for testing
/// Tests that we can modify different macros in the same file
pub fn get_first() -> code_cell::CodeCell<u32> {
    code_cell::code_cell(100u32)
}

pub fn get_second() -> code_cell::CodeCell<u32> {
    code_cell::code_cell(20u32)
}

pub fn get_third() -> code_cell::CodeCell<u32> {
    code_cell::code_cell(30u32)
}
