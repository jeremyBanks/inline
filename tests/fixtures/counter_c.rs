/// Fixture C: Counter that can be modified by tests
/// Default value: 0
pub fn get() -> code_cell::CodeCell<u32> {
    code_cell::code_cell(0u32)
}
