use super::{
    block::Block,
    expr::Expr,
    func::FuncDef,
    stmts::{ConstDef, Stmt, VarDef},
    unit::CompilationUnit,
};

pub trait AstVisitor {
    type Output;
    fn block(&mut self, block: &Block) -> Self::Output;
    fn comment(&mut self) -> Self::Output;
    fn expr(&mut self, expr: &Expr) -> Self::Output;
    fn const_def(&mut self, stmt: &ConstDef) -> Self::Output;
    fn var_def(&mut self, stmt: &VarDef) -> Self::Output;
    fn func_def(&mut self, stmt: &FuncDef) -> Self::Output;

    #[inline]
    fn comp_unit(&mut self, unit: &CompilationUnit) -> Self::Output {
        self.block(&unit.block)
    }

    #[inline]
    fn stmt(&mut self, stmt: &Stmt) -> Self::Output {
        match stmt {
            Stmt::Comment(_) => self.comment(),
            Stmt::Const(stmt) => self.const_def(stmt),
            Stmt::Var(stmt) => self.var_def(stmt),
            Stmt::Expr(expr) => self.expr_stmt(expr),
            Stmt::Func(func) => self.func_def(func),
        }
    }

    #[inline]
    fn expr_stmt(&mut self, expr: &Expr) -> Self::Output {
        self.expr(expr)
    }
}
