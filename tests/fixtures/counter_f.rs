/// Fixture F: Counter for testing formatting preservation
/// Default value: 0
pub fn get() -> code_cell::CodeCell<u32> {
    // Some comment before
    code_cell::code_cell(0u32)  // Trailing comment
}
