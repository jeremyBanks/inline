/// Fixture: Config string that can be modified by tests
/// Default value: "default"
pub fn get() -> inline::InlineCell<String> {
    inline::cell("default" . to_owned ())
}
