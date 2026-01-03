/// Fixture F: Counter for testing formatting preservation
/// Default value: 0
pub fn get() -> litter::Litter<u32> {
    // Some comment before
    litter::litter!(0u32)  // Trailing comment
}
