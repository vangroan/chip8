//! Statement parsing.
use super::expr::Expr;
use crate::{tokens::Token, trivia::SyntaxTrivia};

#[derive(Debug)]
pub enum Stmt {
    Const(ConstDef),
    /// Variable definition
    Var(VarDef),
    /// Expression Statements
    Expr(Expr),
}

/// Definition of constant value.
///
/// # Example
///
/// ```text
/// const FOO = 1;
/// ```
#[derive(Debug)]
pub struct ConstDef {
    pub keyword: Token,
    pub name: String,
    pub trail: SyntaxTrivia,
}

#[derive(Debug)]
pub struct VarDef {
    pub keyword: Token,
    pub name: String,
    pub trail: SyntaxTrivia,
}
