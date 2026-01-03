/// Fixture E: Counter for testing concurrent modification detection
/// Default value: 0
pub fn get() -> litter::Litter<u32> {
    litter::litter!(0u32)
}
