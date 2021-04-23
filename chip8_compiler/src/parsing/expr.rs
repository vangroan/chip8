use super::{ident::Ident, literal::Literal, Parse, ParseError};
use crate::{
    token_stream::{TokenError, TokenStream},
    tokens::{Token, TokenKind},
};
use std::{convert::TryFrom, fmt, ops};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
enum Precedence {
    /// Tokens that terminate an expression
    /// should have a precedence of `None`.
    None = 0,
    Lowest = 1,
    Assignment = 2,    // =
    Conditional = 3,   // ?:
    LogicalOr = 4,     // ||
    LogicalAnd = 5,    // &&
    Equality = 6,      // == !=
    Is = 7,            // is
    Comparison = 8,    // < > <= >=
    BitwiseOr = 9,     // |
    BitwiseXor = 10,   // ^
    BitwiseAnd = 11,   // &
    BitwiseShift = 12, // << >>
    Range = 13,        // .. ...
    Term = 14,         // + -
    Factor = 15,       // * / %
    Unary = 16,        // - ! ~
    Call = 17,         // . () []
    Primary = 18,
}

impl From<i32> for Precedence {
    #[rustfmt::skip]
    fn from(value: i32) -> Self {
        use Precedence as P;
        match value {
            0  => P::None,
            1  => P::Lowest,
            2  => P::Assignment,
            3  => P::Conditional,
            4  => P::LogicalOr,
            5  => P::LogicalAnd,
            6  => P::Equality,
            7  => P::Is,
            8  => P::Comparison,
            9  => P::BitwiseOr,
            10 => P::BitwiseXor,
            11 => P::BitwiseAnd,
            12 => P::BitwiseShift,
            13 => P::Range,
            14 => P::Term,
            15 => P::Factor,
            16 => P::Unary,
            17 => P::Call,
            18 => P::Primary,
            _  => P::None,
        }
    }
}

impl fmt::Display for Precedence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.as_i32(), f)
    }
}

impl ops::Add<i32> for Precedence {
    type Output = Precedence;

    fn add(self, rhs: i32) -> Self::Output {
        Precedence::try_from(self.as_i32() + rhs).unwrap()
    }
}

impl Precedence {
    /// Convert the precedence to an integer value.
    #[inline(always)]
    fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// Get the precedence of the given token type in the context
    /// of the expression parser.
    fn of(token_kind: TokenKind) -> Precedence {
        use TokenKind as T;

        match token_kind {
            T::Number => Precedence::Lowest,
            T::Plus | T::Minus => Precedence::Term,
            T::Star | T::Slash => Precedence::Factor,
            T::Eq => Precedence::Assignment,
            _ => Precedence::None,
        }
    }
}

/// Associativity is the precedence tie-breaker.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Associativity {
    Left,
    Right,
}

impl Associativity {
    fn of(token_kind: TokenKind) -> Associativity {
        if token_kind == TokenKind::Eq {
            Associativity::Right
        } else {
            Associativity::Left
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    /// For development
    NoOp,
    Unary(UnaryOp),
    Binary(BinaryOp),
    Literal(Literal),
    /// Variable or constant value access.
    Access(Access),
}

/// Arithmetic operation with an expression on the right side.
#[derive(Debug)]
pub struct UnaryOp {
    pub operator: Token,
    pub rhs: Box<Expr>,
}

/// Arithmetic operation with an expression on either side.
#[derive(Debug)]
pub struct BinaryOp {
    pub operator: Token,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

/// Access a constant, variable or function, by name.
#[derive(Debug)]
pub struct Access {
    pub ident: Ident,
}

impl Parse for Expr {
    type Output = Self;
    type Err = ParseError;

    fn parse(input: &mut TokenStream) -> Result<Self, ParseError> {
        // Expression parser is seeded with the lowets
        // precedence, but not the `None` precedence.
        //
        // This allows token with the `None` precedence
        // to terminate the parser.
        Self::parse_precedence(input, Precedence::Lowest)
    }
}

/// Recursive parsing methods.
impl Expr {
    /// Entrypoint for the top-down precedence parser.
    ///
    /// The implementation is a straight forward Pratt parser.
    fn parse_precedence(input: &mut TokenStream, precedence: Precedence) -> Result<Expr, ParseError> {
        // The current expression node is wrapped in `Option`
        // so that it can be moved into the recursive parser,
        // and the stack value swapped with the parsing result.
        let mut left = Some(Self::parse_prefix(input)?);

        input.reset_peek();

        // Recurse down the right side of each expression until we
        // encounter an node with higher precedence.
        while precedence <= input.peek().map(|t| Precedence::of(t.kind)).unwrap_or(Precedence::None) {
            // Peek advances a peek pointer inside the token stream,
            // so it needs to be reset otherwise we are inadvertently
            // looking further ahead by one token.
            input.reset_peek();

            // There is no expression right of the last one, so we
            // just return what we have.
            if let Ok(TokenKind::EOS) | Err(TokenError::EndOfSource) = input.peek().map(|token| token.kind) {
                return Ok(left.take().unwrap());
            }

            let token = input.next_token().ok_or(ParseError::EOS)??;
            left = Some(Self::parse_infix(input, left.take().unwrap(), token)?);

            // Prepare for next iteration.
            input.reset_peek();
        }

        // Higher precedence was encountered.
        Ok(left.take().unwrap())
    }

    /// Parse next token as a prefix in an expression.
    ///
    /// This function is analogous to a parselet.
    fn parse_prefix(input: &mut TokenStream) -> Result<Expr, ParseError> {
        use TokenKind as T;

        input.reset_peek();
        match input.peek().map(|t| t.kind)? {
            T::Number => {
                // Only one number type.
                Literal::parse(input).map(Expr::Literal)
            }
            T::Ident => {
                // Constant, variable or function call in expression.
                Ok(Expr::Access(Access {
                    ident: Ident::parse(input)?,
                }))
            }
            T::Minus => {
                // Negate
                Ok(Expr::Unary(UnaryOp {
                    operator: input.consume(T::Minus)?,
                    rhs: Self::parse_precedence(input, Precedence::Unary).map(Box::new)?,
                }))
            }
            T::Plus => {
                // Prefix plus is allowed but does nothing when executed.
                Ok(Expr::Unary(UnaryOp {
                    operator: input.consume(T::Plus)?,
                    rhs: Self::parse_precedence(input, Precedence::Unary).map(Box::new)?,
                }))
            }
            token_kind => Err(ParseError::Token(TokenError::Unexpected {
                encountered: token_kind,
                msg: "prefix expression expected".to_owned(),
            })),
        }
    }

    /// Parse an infix, postfix or mixfix operator.
    ///
    /// This function is analogous to a parselet.
    ///
    /// Includes non-obvious tokens like opening parentheses `(`.
    fn parse_infix(input: &mut TokenStream, left: Expr, token: Token) -> Result<Expr, ParseError> {
        use TokenKind as T;

        let precedence = Precedence::of(token.kind);

        // Associativity is handled by adjusting the precedence.
        // Left associativity is achieved by increasing the precedence
        // by 1. This increases the threshold that any infix expressions
        // to our right must exceed.
        //
        // Right associativity can be achieved by keeping
        // the precedence the same, thus keeping the threshold any
        // subsequent infix expression need to exceed to be parsed.
        //
        // Wren doesn't appear to have right-associative operators.
        let binding_power = if Associativity::of(token.kind) == Associativity::Left {
            1
        } else {
            0
        };

        // Recurse back into expression parser to handle
        // the right hand side.
        //
        // The left hand side will wait for us here on
        // the call stack.
        let right = Self::parse_precedence(input, precedence + binding_power)?;

        match token.kind {
            T::Plus | T::Minus | T::Star | T::Slash => {
                // Binary operations.
                Ok(Expr::Binary(BinaryOp {
                    operator: token,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }))
            }
            token_kind => Err(ParseError::Token(TokenError::Unexpected {
                encountered: token_kind,
                msg: "unknown infix expression kind".to_owned(),
            })),
        }
    }
}
