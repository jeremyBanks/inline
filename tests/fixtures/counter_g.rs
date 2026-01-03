/// Fixture G: Multiple counters in the same file for testing
/// Tests that we can modify different macros in the same file
pub fn get_first() -> inline::Inline<u32> {
    inline::inline!(10u32)
}

pub fn get_second() -> inline::Inline<u32> {
    inline::inline!(20u32)
}

pub fn get_third() -> inline::Inline<u32> {
    inline::inline!(30u32)
}
