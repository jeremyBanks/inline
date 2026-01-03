/// Fixture F: Counter for testing formatting preservation
/// Default value: 0
pub fn get() -> inline::Inline<u32> {
    // Some comment before
    inline::inline!(0u32)  // Trailing comment
}
