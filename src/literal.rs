// Re-export databake's Bake trait for serializing values to Rust source code
pub use databake::Bake;

/// Trait alias for types that can be used with `literal!()`.
///
/// This trait combines three requirements:
/// - `Bake`: Serialize to Rust source code (from databake crate)
/// - `Clone`: Copy values for change detection (write-on-drop)
/// - `PartialEq`: Detect mutations by comparing original vs current
///
/// All types implementing these three traits automatically implement `Value`
/// via the blanket implementation below.
pub trait Value: Bake + Clone + PartialEq {}

// Blanket implementation: any type that is Bake + Clone + PartialEq is a Value
impl<T> Value for T where T: Bake + Clone + PartialEq {}
