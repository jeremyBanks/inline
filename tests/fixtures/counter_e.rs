/// Fixture E: Counter for testing concurrent modification detection
/// Default value: 0
pub fn get() -> jeb_literal::Literal<u32> {
    jeb_literal::literal!(777u32)  // Externally modified!
}
