/// Fixture E: Counter for testing concurrent modification detection
/// Default value: 0
pub fn get() -> code_cell::CodeCell<u32> {
    code_cell::code_cell(777u32)  // Externally modified!
}
