mod ir;
mod symbol;

pub use ir::IR;
pub use symbol::{Symbol, SymbolRealm, SymbolTable, SymbolType};

use crate::parsing::{AstVisitor, Block, CompilationUnit, ConstDef, Expr, Stmt, VarDef};
use smol_str::SmolStr;
use std::{collections::VecDeque, error, fmt};

pub struct CodeGen {
    code: Vec<IR>,
    next_id: usize,

    scopes: VecDeque<SymbolTable>,

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

            constants: vec![],
            variables: vec![],
        }
    }

    #[inline]
    pub fn compile(&mut self, unit: &CompilationUnit) {
        self.comp_unit(unit);
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

    fn add_var(&mut self, name: &str) -> Result<u8, CompileError> {
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

    fn create_register(&mut self) -> Register {
        Register { id: self.next_id() }
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
    fn block(&mut self, block: &Block) -> Result<(), CompileError> {
        for stmt in &block.stmts {
            self.stmt(stmt);
        }

        Ok(())
    }

    fn comment(&mut self) -> Result<(), CompileError> {
        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::NoOp => {
                todo!()
            }
            _ => todo!(),
        }
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
        if let Some(scope) = self.scopes.front_mut() {
            if scope.contains_symbol(var_def.name.as_str()) {
                Err(CompileError::SymbolExists)
            } else {
                scope.add_symbol(Symbol {
                    name: var_def.name.clone(),
                    realm: SymbolRealm::Var,
                    // TODO: Constant type
                    ty: SymbolType::U8,
                });
                Ok(())
            }
        } else {
            Err(CompileError::NoScope)
        }
    }

    #[inline]
    fn comp_unit(&mut self, unit: &CompilationUnit) -> Result<(), CompileError> {
        self.block(&unit.block)
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
        self.expr(expr)
    }
}

#[derive(Debug, Clone)]
struct Register {
    id: usize,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "r{}", self.id)
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
