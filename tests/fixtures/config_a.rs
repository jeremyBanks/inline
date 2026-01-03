/// Fixture: Config string that can be modified by tests
/// Default value: "default"
pub fn get() -> &'static parking_lot::Mutex<inline::Inline<String>> {
    inline::inline!("default" . to_owned ())
}
