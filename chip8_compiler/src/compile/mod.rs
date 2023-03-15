pub mod codegen;
mod consteval;
mod ir;
mod mapper;
mod register;
mod symbol;

pub use ir::{assemble, IR};
pub use mapper::Mapper;
use register::{Register, RegisterMask};
pub use symbol::{Symbol, SymbolKind, SymbolTable, ValueType};

use crate::{
    parsing::{Block, CompilationUnit, ConstDef, Expr, FuncDef, LitValue, Literal, Stmt, VarDef},
    tokens::TokenKind,
};
use smol_str::SmolStr;
use std::{error, fmt};

#[deprecated]
pub struct CodeGen {
    pub code: Vec<IR>,
    /// Mask to track which of the 15
    /// operand registers are occupied.
    pub mask: RegisterMask,
    /// Symbols contain information like
    /// types and reserved registers for
    /// variables.
    pub symbols: SymbolTable,
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
            mask: RegisterMask::default(),
            symbols: SymbolTable::default(),
        }
    }

    #[inline]
    pub fn compile(
        &mut self,
        unit: &CompilationUnit,
        _symbols: &SymbolTable,
    ) -> Result<Vec<u8>, CompileError> {
        println!("std::mem::size_of::<IR>() -> {}", std::mem::size_of::<IR>());
        self.emit_comp_unit(unit)?;
        Ok(assemble(&self.code))
    }

    #[inline]
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        todo!()
    }

    fn add_var(&mut self, name: &str) -> Result<Register, CompileError> {
        // if self.variables.len() > Self::MAX_VARIABLES {
        //     Err(CompileError::RegisterOverflow)
        // } else {
        //     // Important to reserve the register for variable
        //     // usage, so it won't be cleared by expressions.
        //     let register = Register {
        //         usage: Usage::Local,
        //         ..self.next_register()?
        //     };
        //     self.variables.push(Var {
        //         name: SmolStr::from(name),
        //         register: register.clone(),
        //     });
        //     Ok(register)
        // }
        todo!()
    }

    fn next_register(&mut self) -> Result<Register, CompileError> {
        self.mask
            .find_vacant()
            .map(|id| Register {
                id,
                ..Default::default()
            })
            .ok_or(CompileError::RegisterOverflow)
    }

    fn emit(&mut self, ir: IR) {
        self.code.push(ir)
    }
}

impl Default for CodeGen {
    #[inline]
    fn default() -> Self {
        CodeGen::new()
    }
}

impl CodeGen {
    #[inline]
    fn emit_comp_unit(&mut self, unit: &CompilationUnit) -> Result<(), CompileError> {
        self.emit_block(&unit.block)
    }

    #[inline]
    fn emit_block(&mut self, block: &Block) -> Result<(), CompileError> {
        for stmt in &block.stmts {
            self.emit_stmt(stmt)?;
        }

        Ok(())
    }

    fn emit_expr(&mut self, expr: &Expr) -> Result<Register, CompileError> {
        match expr {
            Expr::NoOp => {
                todo!()
            }
            Expr::Literal(Literal {
                value: LitValue::U8(val),
                ..
            }) => self.emit_const_u8(*val, None),
            Expr::Binary(expr) => {
                let vx = self.emit_expr(&expr.lhs)?;
                let vy = self.emit_expr(&expr.rhs)?;

                match expr.operator.kind {
                    TokenKind::Plus => self.emit(IR::MathAdd(vx.id, vy.id)),
                    token_kind => panic!("invalid expression token kind {:?}", token_kind),
                }

                // If the right hand side's register is a temporary value,
                // then we can clear it after it's used in computation.
                // This expression is the owner of the sub-expression's
                // register.
                //
                // register is not temporary when it belongs to a variable.
                // This expression is thus not allowed to remove the register.
                if vy.is_temp() {
                    self.mask.remove(vy.id);
                }

                Ok(vx)
            }
            _ => todo!(),
        }
    }

    fn emit_const_u8(
        &mut self,
        value: u8,
        result: Option<Register>,
    ) -> Result<Register, CompileError> {
        let vx = match result {
            Some(r) => r,
            None => self.next_register()?,
        };
        self.emit(IR::SetConst(vx.id, value));
        Ok(vx)
    }

    /// Move value from one register to another.
    ///
    /// If the source register is temporary, it will be
    /// freed in the mask.
    fn emit_move(&mut self, src: &Register, dest: &Register) {
        self.emit(IR::Assign(dest.id, src.id));
        if src.is_temp() {
            self.mask.remove(src.id);
        }
    }

    fn const_def(&mut self, const_def: &ConstDef) -> Result<(), CompileError> {
        // TODO: RHS - const expr
        // if let Some(scope) = self.scopes.front_mut() {
        //     if scope.contains_symbol(const_def.name.as_str()) {
        //         Err(CompileError::SymbolExists)
        //     } else {
        //         scope.add_symbol(Symbol {
        //             name: const_def.name.clone(),
        //             realm: SymbolRealm::Const,
        //             // TODO: Constant type
        //             ty: SymbolType::U8,
        //         });
        //         Ok(())
        //     }
        // } else {
        //     Err(CompileError::NoScope)
        // }
        todo!()
    }

    fn emit_var_def(&mut self, var_def: &VarDef) -> Result<(), CompileError> {
        // if let Some(scope) = self.scopes.front_mut() {
        //     if scope.contains_symbol(var_def.name.as_str()) {
        //         Err(CompileError::SymbolExists)
        //     } else {
        //         scope.add_symbol(Symbol {
        //             name: var_def.name.clone(),
        //             realm: SymbolRealm::Var,
        //             // TODO: Constant type
        //             ty: SymbolType::U8,
        //         });
        //         Ok(())
        //     }
        // } else {
        //     Err(CompileError::NoScope)
        // }
        // Expression result is will be in the
        // return register.
        // TODO: Variable needs a symbol in the symbol table with
        //       its own reserved register.
        let r = self.next_register()?;

        let vx = match var_def.rhs {
            Some(ref expr) => self.emit_expr(expr)?,
            None => self.emit_const_u8(0, Some(r.clone()))?,
        };
        println!("Variable '{}' in Register V{}", var_def.name, vx.id);

        // TODO: We can save on one register if we manage to reuse
        //       the variable's register for the expression's result.
        if r != vx {
            self.emit_move(&vx, &r);
        }

        Ok(())
    }

    fn emit_func_def(&mut self, _func: &FuncDef) -> Result<(), CompileError> {
        todo!()
    }

    #[inline]
    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Comment(_) => Ok(()),
            Stmt::Const(stmt) => self.const_def(stmt),
            Stmt::Var(stmt) => self.emit_var_def(stmt),
            Stmt::Expr(expr) => self.expr_stmt(expr),
            Stmt::Func(func) => self.emit_func_def(func),
        }
    }

    #[inline]
    fn expr_stmt(&mut self, expr: &Expr) -> Result<(), CompileError> {
        self.emit_expr(expr).map(|_| ())
    }
}

struct Const {
    name: SmolStr,
    value: u8,
}

struct Var {
    name: SmolStr,
    register: Register,
}

#[derive(Debug)]
pub enum CompileError {
    /// FIXME: NoOp error just so we can convert Option to Result :[
    NoOp,
    NoScope,
    RegisterOverflow,
    SymbolExists,
    SymbolDoesNotExist,
    NotConst,
    ConstEval,
    UnsupportOperation,
    TypeError,
}

impl error::Error for CompileError {}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompileError::RegisterOverflow => write!(
                f,
                "number of variables exceed number of available registers {}",
                Self::RegisterOverflow
            ),
            _ => write!(f, "compile error"),
        }
    }
}
