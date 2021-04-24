mod ir;
mod register;
mod symbol;

pub use ir::IR;
use register::{Register, RegisterMask};
pub use symbol::{Symbol, SymbolRealm, SymbolTable, SymbolType};

use crate::{
    parsing::{AstVisitor, Block, CompilationUnit, ConstDef, Expr, LitValue, Literal, Stmt, VarDef},
    tokens::TokenKind,
};
use smol_str::SmolStr;
use std::{collections::VecDeque, error, fmt};

pub struct CodeGen {
    pub code: Vec<IR>,
    next_id: usize,

    scopes: VecDeque<SymbolTable>,
    pub mask: RegisterMask,

    constants: Vec<Const>,
    variables: Vec<Var>,
}

impl CodeGen {
    /// Maximum number of variables allowed in a function.
    /// Limited by number of available operand registers
    /// in Chip-8.
    pub const MAX_VARIABLES: usize = 15;

    #[inline]
    pub fn new() -> Self {
        Self {
            code: vec![],
            next_id: 0,

            // Initialize default global scope.
            scopes: VecDeque::from(vec![SymbolTable::default()]),
            mask: RegisterMask::default(),

            constants: vec![],
            variables: vec![],
        }
    }

    #[inline]
    pub fn compile(&mut self, unit: &CompilationUnit) -> Result<(), CompileError> {
        self.comp_unit(unit)
    }

    #[inline]
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        todo!()
    }

    fn add_const(&mut self, name: &str, value: u8) {
        self.constants.push(Const {
            name: SmolStr::from(name),
            value,
        });
    }

    fn add_var(&mut self, name: &str) -> Result<Register, CompileError> {
        if self.variables.len() > Self::MAX_VARIABLES {
            Err(CompileError::RegisterOverflow)
        } else {
            let index = self.variables.len();
            self.variables.push(Var {
                name: SmolStr::from(name),
            });
            Ok(index as u8)
        }
    }

    fn next_register(&mut self) -> Result<Register, CompileError> {
        self.mask.find_vacant().ok_or(CompileError::RegisterOverflow)
    }

    fn emit(&mut self, ir: IR) {
        self.code.push(ir)
    }

    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
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
    fn comp_unit(&mut self, unit: &CompilationUnit) -> Result<(), CompileError> {
        self.block(&unit.block)
    }

    #[inline]
    fn block(&mut self, block: &Block) -> Result<(), CompileError> {
        for stmt in &block.stmts {
            self.stmt(stmt)?;
        }

        Ok(())
    }

    fn comment(&mut self) -> Result<(), CompileError> {
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
            }) => {
                self.emit_const_u8(*val)
            }
            Expr::Binary(expr) => {
                let vx = self.emit_expr(&expr.lhs)?;
                let vy = self.emit_expr(&expr.rhs)?;

                match expr.operator.kind {
                    TokenKind::Plus => self.emit(IR::MathAdd(vx, vy)),
                    token_kind => panic!("invalid expression token kind {:?}", token_kind),
                }

                self.mask.remove(vy);
                Ok(vx)
            }
            _ => todo!(),
        }
    }

    fn emit_const_u8(&mut self, value: u8) -> Result<Register, CompileError> {
        let vx = self.next_register()?;
        self.emit(IR::SetConst(vx, value));
        Ok(vx)
    }

    fn const_def(&mut self, const_def: &ConstDef) -> Result<(), CompileError> {
        // TODO: RHS - const expr
        if let Some(scope) = self.scopes.front_mut() {
            if scope.contains_symbol(const_def.name.as_str()) {
                Err(CompileError::SymbolExists)
            } else {
                scope.add_symbol(Symbol {
                    name: const_def.name.clone(),
                    realm: SymbolRealm::Const,
                    // TODO: Constant type
                    ty: SymbolType::U8,
                });
                Ok(())
            }
        } else {
            Err(CompileError::NoScope)
        }
    }

    fn var_def(&mut self, var_def: &VarDef) -> Result<(), CompileError> {
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
        let vx = match var_def.rhs {
            Some(ref expr) => self.emit_expr(expr)?,
            None => self.emit_const_u8(0)?,
        };
        println!("Variable '{}' in Register V{}", var_def.name, vx);

        Ok(())
    }



    #[inline]
    fn stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Comment => self.comment(),
            Stmt::Const(stmt) => self.const_def(stmt),
            Stmt::Var(stmt) => self.var_def(stmt),
            Stmt::Expr(expr) => self.expr_stmt(expr),
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
}

#[derive(Debug)]
pub enum CompileError {
    NoScope,
    RegisterOverflow,
    SymbolExists,
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
