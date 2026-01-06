/// Fixture: Config string that can be modified by tests
/// Default value: "default"
pub fn get() -> jeb_literal::Literal<String> {
    jeb_literal::literal("default" . to_owned ())
}
