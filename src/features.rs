#[cfg(all(
    feature = "panic",
    feature = "forbid-panic",
    not(feature = "__all_features__")
))]
compile_error!("`inline` crate flags `panic` and `forbid-panic` are both enabled.");

#[cfg(all(
    feature = "mut",
    feature = "forbid-mut",
    not(feature = "__all_features__")
))]
compile_error!("`inline` crate flags `mut` and `forbid-mut` are both enabled.");

#[cfg(all(
    feature = "write",
    feature = "forbid-write",
    not(feature = "__all_features__")
))]
compile_error!("`inline` crate flags `write` and `forbid-write` are both enabled.");

#[cfg(all(
    feature = "serde",
    feature = "forbid-serde",
    not(feature = "__all_features__")
))]
compile_error!("`inline` crate flags `serde` and `forbid-serde` are both enabled.");
