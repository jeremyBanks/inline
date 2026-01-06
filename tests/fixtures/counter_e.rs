/// Fixture E: Counter for testing concurrent modification detection
/// Default value: 0
pub fn get() -> inline::InlineCell<u32> {
    inline::cell(777u32)  // Externally modified!
}
