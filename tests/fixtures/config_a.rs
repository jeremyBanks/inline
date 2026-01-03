/// Fixture: Config string that can be modified by tests
/// Default value: "default"
pub fn get() -> litter::Litter<String> {
    litter::litter!("default".to_owned())
}
