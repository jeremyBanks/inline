/// Fixture G: Multiple counters in the same file for testing
/// Tests that we can modify different macros in the same file
pub fn get_first() -> litter::Litter<u32> {
    litter::litter!(10u32)
}

pub fn get_second() -> litter::Litter<u32> {
    litter::litter!(20u32)
}

pub fn get_third() -> litter::Litter<u32> {
    litter::litter!(30u32)
}
