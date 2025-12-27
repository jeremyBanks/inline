// Re-export databake's Bake trait as our "Literal" concept
// This allows any type implementing Bake to be used with litter

pub use databake::Bake;

// For compatibility, we define a Literal trait that requires Bake + Clone
// We compare values by comparing their baked tokens, not by PartialEq
pub trait Literal: Bake + Clone {}

// Blanket implementation: any type that is Bake + Clone is a Literal
impl<T> Literal for T where T: Bake + Clone {}
