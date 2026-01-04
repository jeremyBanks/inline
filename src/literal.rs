// Re-export databake's Bake trait as our "Value" concept
// This allows any type implementing Bake to be used with literal values

pub use databake::Bake;

// We define a Value trait that requires Bake + Clone
// Clone is needed for the write-on-drop functionality (Priority #2)
// We compare values by comparing their baked tokens, not by PartialEq
pub trait Value: Bake + Clone {}

// Blanket implementation: any type that is Bake + Clone is a Value
impl<T> Value for T where T: Bake + Clone {}
