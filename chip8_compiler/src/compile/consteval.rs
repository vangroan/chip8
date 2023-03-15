// Constant expression evaluator.
use super::{
    mapper::Mapper,
    symbol::{ConstValue, SymbolKind},
    CompileError,
};
use crate::{
    parsing::{Expr, LitValue},
    tokens::TokenKind,
};

/// Constant expression evaluator.
///
/// Constant values are evaluated by the compiler, and
/// a fixed compile-time value is inserted into the symbol
/// table.
///
/// The implemenation is a simple tree walker that handles
/// a subset of the abstract syntax tree, resulting in a
/// concrete value.
///
/// Evaluator is dynamically typed for simplicity.
pub struct ConstEval<'a> {
    /// Evaluator is not allowed to insert new symbols
    /// into the table, or push new scopes, it may only
    /// access existing const symbols in the current scope.
    ///
    /// The language doesn't allow a constant expression to
    /// have side-effects.
    symbols: &'a Mapper,
}

impl<'a> ConstEval<'a> {
    #[inline]
    pub fn new(symbols: &'a Mapper) -> Self {
        Self { symbols }
    }

    /// Entry point for the evaluator.
    pub fn eval_expr(&self, expr: &Expr) -> Result<ConstValue, CompileError> {
        use ConstValue as V;

        match expr {
            Expr::Literal(lit) => {
                // Simple case of a literal value.
                match lit.value {
                    LitValue::Bool(val) => Ok(V::Bool(val)),
                    LitValue::U8(val) => Ok(V::U8(val)),
                }
            }
            Expr::Access(access_expr) => {
                // When a constant expression accesses a symbol, we ensure
                // that it has been defined and the symbol must be constant
                // as well.
                //
                // Constant expression cannot access a runtime value.
                self.symbols
                    .lookup_symbol(access_expr.ident.name.as_str())
                    .ok_or(CompileError::SymbolDoesNotExist)
                    .and_then(|symbol| match symbol.kind {
                        SymbolKind::Const(ref value) => Ok(value.clone()),
                        _ => Err(CompileError::NotConst),
                    })
            }
            Expr::Binary(bin_expr) => {
                let lhs = self.eval_expr(&bin_expr.lhs)?;
                let rhs = self.eval_expr(&bin_expr.rhs)?;

                match bin_expr.operator.kind {
                    TokenKind::Plus => match (lhs, rhs) {
                        (V::U8(a), V::U8(b)) => Ok(V::U8(a + b)),
                        (V::Bool(_), V::Bool(_)) => Err(CompileError::UnsupportOperation),
                        _ => Err(CompileError::TypeError),
                    },
                    TokenKind::Minus => match (lhs, rhs) {
                        (V::U8(a), V::U8(b)) => Ok(V::U8(a - b)),
                        (V::Bool(_), V::Bool(_)) => Err(CompileError::UnsupportOperation),
                        _ => Err(CompileError::TypeError),
                    },
                    _ => Err(CompileError::UnsupportOperation),
                }
            }
            _ => Err(CompileError::UnsupportOperation),
        }
    }
}
