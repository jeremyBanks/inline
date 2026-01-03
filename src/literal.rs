// Re-export databake's Bake trait as our "Value" concept
// This allows any type implementing Bake to be used with literal values

pub use databake::Bake;

// We define a Value trait that only requires Bake
// We compare values by comparing their baked tokens, not by PartialEq
// We don't need Clone because we move values in set() instead of cloning
pub trait Value: Bake {}

// Blanket implementation: any type that is Bake is a Value
impl<T> Value for T where T: Bake {}
