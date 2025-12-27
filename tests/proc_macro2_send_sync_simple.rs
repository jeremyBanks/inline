#[test]
fn proc_macro2_should_be_send_sync() {
    // proc_macro2 is designed to be Send+Sync outside of proc macro context
    // This is its whole purpose!

    fn is_send<T: Send>() -> bool { true }
    fn is_sync<T: Sync>() -> bool { true }

    assert!(is_send::<proc_macro2::TokenStream>());
    assert!(is_sync::<proc_macro2::TokenStream>());
    assert!(is_send::<proc_macro2::Span>());
    assert!(is_sync::<proc_macro2::Span>());

    println!("✓ proc_macro2 types are Send+Sync as expected");
}
