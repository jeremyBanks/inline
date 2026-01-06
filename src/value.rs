// Re-export databake's Bake trait as our "Value" concept
// This allows any type implementing Bake to be used with literal values

pub use databake::Bake;

// We define a Value trait that requires Bake + Clone + PartialEq
// Clone is needed for the write-on-drop functionality
// PartialEq is used for change detection
// Bake is used for serialization to source code
pub trait Value: Bake + Clone + PartialEq {}

// Blanket implementation: any type that is Bake + Clone + PartialEq is a Value
impl<T> Value for T where T: Bake + Clone + PartialEq {}
