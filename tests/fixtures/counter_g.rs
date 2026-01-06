/// Fixture G: Multiple counters in the same file for testing
/// Tests that we can modify different macros in the same file
pub fn get_first() -> inline::InlineCell<u32> {
    inline::cell(100u32)
}

pub fn get_second() -> inline::InlineCell<u32> {
    inline::cell(20u32)
}

pub fn get_third() -> inline::InlineCell<u32> {
    inline::cell(30u32)
}
