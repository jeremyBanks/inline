// Re-export databake's Bake trait as our "Literal" concept
// This allows any type implementing Bake to be used with litter

pub use databake::Bake;

// For compatibility, we define a Literal trait that requires Bake + other useful traits
pub trait Literal: Bake + PartialEq + Clone {}

// Blanket implementation: any type that is Bake + PartialEq + Clone is a Literal
impl<T> Literal for T where T: Bake + PartialEq + Clone {}
