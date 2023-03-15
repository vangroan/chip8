use super::{
    consteval::ConstEval,
    ir::{assemble, IR},
    mapper::Mapper,
    register::RegisterMask,
    symbol::{Symbol, SymbolKind, ValueType},
    CompileError,
};
use crate::parsing::{Block, ConstDef, DefStmt, Expr, Prog, Stmt, VarDef};
use std::convert::TryFrom;

type CompileResult = Result<(), CompileError>;

/// Code generator.
pub struct CodeGen {
    /// Resulting generated code.
    code: Vec<IR>,
    /// Mask to track which of the 15
    /// operand registers are occupied.
    mask: RegisterMask,
    /// Stack of scopes that contain symbol mappings.
    symbols: Mapper,
}

impl CodeGen {
    /// Maximum number of variables allowed in a function.
    /// Limited by number of available operand registers
    /// in CHIP-8.
    pub const MAX_VARIABLES: usize = 15;

    #[inline]
    pub fn new() -> Self {
        Self {
            code: vec![],
            mask: Default::default(),
            symbols: Default::default(),
        }
    }

    pub fn compile(&mut self, prog: &Prog) -> Result<Box<[u8]>, CompileError> {
        self.emit_program(prog)?;
        Ok(assemble(&self.code).into_boxed_slice())
    }

    /// Clear the internal state so the code generator can be reused.
    pub fn reset(&mut self) {
        self.code.clear();
        self.mask = Default::default();
        self.symbols.reset();
    }
}

/// Recursive visitor
impl CodeGen {
    fn emit_program(&mut self, prog: &Prog) -> CompileResult {
        for stmt in &prog.stmts {
            self.emit_def_stmt(stmt)?;
        }
        Ok(())
    }

    fn emit_block(&mut self, block: &Block) -> CompileResult {
        for stmt in &block.stmts {
            self.emit_stmt(stmt)?;
        }
        Ok(())
    }

    fn emit_def_stmt(&mut self, def_stmt: &DefStmt) -> CompileResult {
        match def_stmt {
            DefStmt::Func(_func_def) => todo!("func def stmt"),
            DefStmt::Stmt(stmt) => self.emit_stmt(stmt),
        }
    }

    #[inline]
    fn emit_stmt(&mut self, stmt: &Stmt) -> CompileResult {
        match stmt {
            Stmt::Comment(_) => Ok(()),
            Stmt::Const(stmt) => self.handle_const_def(stmt),
            Stmt::Var(stmt) => self.handle_var_def(stmt),
            Stmt::Expr(_expr) => todo!("expr stmt"),
            Stmt::Func(_func) => todo!("func stmt"),
        }
    }

    /// Load constant definition into symbol table.
    ///
    /// Constants only exist in a symbol table until they are accessed.
    /// later when accessed via runtime expressions, the constant will be
    /// loaded into a register chosen by a function, but not here.
    ///
    /// The right-hand-side of a constant definition can contain some
    /// simple expressions, evaluated at compile time to a single value.
    /// See [`compile::consteval::ConstEval`].
    fn handle_const_def(&mut self, const_def: &ConstDef) -> CompileResult {
        if self.symbols.check_exists(const_def.name.as_str()) {
            return Err(CompileError::SymbolExists);
        }

        // Before adding the symbol to the table, evaluate the right-hand-side
        // for a compile time value.
        let value = ConstEval::new(&self.symbols).eval_expr(&const_def.rhs)?;

        self.symbols.insert_symbol(
            const_def.name.as_str(),
            Symbol {
                name: const_def.name.clone(),
                kind: SymbolKind::Const(value),
                ty: ValueType::try_from(&const_def.ty.ty)
                    .unwrap_or_else(|_| panic!("unknown type {}", const_def.ty.ty)),
            },
        );

        Ok(())
    }

    /// Load variable definition into the symbol table.
    fn handle_var_def(&mut self, var_def: &VarDef) -> CompileResult {
        if self.symbols.check_exists(var_def.name.as_str()) {
            return Err(CompileError::SymbolExists);
        }

        // TODO: Retrieve next vacant register
        // TODO: Emit runtime bytecode for right-hand-side
        // TODO: Emit load into variable's register
        // TODO: Store variable's register in symbol table

        Ok(())
    }
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}