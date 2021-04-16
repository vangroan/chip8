use crate::tokens::Token;

#[derive(Debug)]
pub struct SyntaxTrivia {
    pub token: Token,
}

pub enum TriviaKind {
    DocComment,
}
