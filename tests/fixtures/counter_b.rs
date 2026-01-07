/// Fixture B: Counter that can be modified by tests
/// Default value: 0
pub fn get() -> inline::InlineCell<u32> {
    inline::cell(0u32)
}
