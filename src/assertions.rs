#[cfg(any(doc, feature = "assertions"))]
pub fn assert_eq<T: PartialEq>(expected: &'static T, actual: &T) -> ! {
    todo!()
}
