/// Fixture C: Counter that can be modified by tests
/// Default value: 0
pub fn get() -> jeb_literal::Literal<u32> {
    jeb_literal::literal!(0u32)
}
