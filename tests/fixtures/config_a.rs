/// Fixture: Config string that can be modified by tests
/// Default value: "default"
pub fn get() -> code_cell::CodeCell<String> {
    code_cell::code_cell("default" . to_owned ())
}
