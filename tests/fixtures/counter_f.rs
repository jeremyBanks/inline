/// Fixture F: Counter for testing formatting preservation
/// Default value: 0
pub fn get() -> inline::InlineCell<u32> {
    // Some comment before
    inline::cell(0u32)  // Trailing comment
}
