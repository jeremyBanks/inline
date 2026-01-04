/// Fixture G: Multiple counters in the same file for testing
/// Tests that we can modify different macros in the same file
pub fn get_first() -> jeb_literal::Literal<u32> {
    jeb_literal::literal!(100u32)
}

pub fn get_second() -> jeb_literal::Literal<u32> {
    jeb_literal::literal!(20u32)
}

pub fn get_third() -> jeb_literal::Literal<u32> {
    jeb_literal::literal!(30u32)
}
