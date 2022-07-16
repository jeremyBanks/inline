#[derive(Debug)]
pub struct LitterHandle<Literal: crate::Literal> {
    literal: Literal,
}
