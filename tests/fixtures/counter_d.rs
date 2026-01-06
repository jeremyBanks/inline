/// Fixture D: Counter for testing multiple modifications
/// Default value: 0
pub fn get() -> inline::InlineCell<u32> {
    inline::cell(0u32)
}
