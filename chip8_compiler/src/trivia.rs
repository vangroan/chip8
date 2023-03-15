use crate::tokens::Token;

#[derive(Debug)]
pub struct SyntaxTrivia {
    pub token: Token,
    pub trail: Option<Box<SyntaxTrivia>>,
}

pub enum TriviaKind {
    Comment,
    NewLine,
}
