/// Fixture F: Counter for testing formatting preservation
/// Default value: 0
pub fn get() -> jeb_literal::Literal<u32> {
    // Some comment before
    jeb_literal::literal!(0u32)  // Trailing comment
}
