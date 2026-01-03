/// Fixture A: Counter that can be modified by tests
/// Default value: 0
pub fn get() -> &'static parking_lot::Mutex<inline::Inline<u32>> {
    inline::inline!(0u32)
}
