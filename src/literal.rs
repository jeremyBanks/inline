// Re-export databake's Bake trait as our "Literal" concept
// This allows any type implementing Bake to be used with inline

pub use databake::Bake;

// For compatibility, we define a Literal trait that only requires Bake
// We compare values by comparing their baked tokens, not by PartialEq
// We don't need Clone because we move values in set() instead of cloning
pub trait Literal: Bake {}

// Blanket implementation: any type that is Bake is a Literal
impl<T> Literal for T where T: Bake {}
