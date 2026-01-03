/// Fixture C: Counter that can be modified by tests
/// Default value: 0
pub fn get() -> litter::Litter<u32> {
    litter::litter!(0u32)
}
