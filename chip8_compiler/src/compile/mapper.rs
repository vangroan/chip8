use super::symbol::{Symbol, SymbolKind, SymbolTable, ValueType};
use crate::compile::symbol::ConstValue;
use crate::parsing::{Access, Block, CompilationUnit, ConstDef, Expr, FuncDef, FuncSig, LitValue, Literal, Stmt, VarDef};
use std::{
    collections::VecDeque,
    convert::{Infallible, TryFrom},
};

/// Builds up a symbol table and maps AST nodes to symbols.
///
/// This task could be done by the code generator, but doing
/// it in its own pass is more readable.
// TODO: Rename to SymbolTable
pub struct Mapper {
    /// Current scope that's being mapped.
    current: Option<SymbolTable>,
    /// Temporary stack of scopes.
    ///
    /// When the current scope starts a sub-scope, the current
    /// scope is pushed onto the stack.
    ///
    /// When the sub-scope ends, it's appended to the
    /// top scope's list of child scopes. The top
    /// is then popped and set as current.
    stack: VecDeque<SymbolTable>,
}

impl Mapper {
    pub fn new() -> Self {
        Mapper {
            // Implicitly the mapper starts with a global scope.
            current: Some(SymbolTable::default()),
            stack: VecDeque::default(),
        }
    }

    /// Clear the internal state of the symbol table so it can be reused.
    pub fn reset(&mut self) {
        self.current = Some(SymbolTable::default());
        self.stack.clear();
    }

    /// Takes the built symbol table and replaces it with an empty global table.
    #[inline]
    fn take_symbols(&mut self) -> SymbolTable {
        assert!(self.stack.is_empty(), "mapping in progress");
        self.current.replace(SymbolTable::default()).unwrap_or_default()
    }

    /// Lookup the given symbol name according to the scope rules.
    ///
    /// First search the current scope for the symbol. If not
    /// found, walk the stack from the top to the bottom.
    ///
    /// This should result in the effect of blocks having access
    /// to their parent scopes, but not their siblings.
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        // First check the current scope
        self.current
            .iter()
            // Then comb the stack from top to bottom.
            .chain(self.stack.iter())
            .find_map(|scope| {
                None.or_else(|| scope.consts.get(name))
                    .or_else(|| scope.vars.get(name))
                    .or_else(|| scope.funcs.get(name))
            })
    }

    /// Insert the given symbol into the currently active scope.
    ///
    /// If a symbol with the given name already exists, it will be
    /// replaced and returned. Otherwise return `None`.
    pub fn insert_symbol(&mut self, name: &str, sym: Symbol) -> Option<Symbol> {
        if let Some(ref mut scope) = self.current {
            match sym.kind {
                SymbolKind::Const(_) => scope.consts.insert(name.to_owned(), sym),
                SymbolKind::Var => scope.vars.insert(name.to_owned(), sym),
                SymbolKind::Function(_) => scope.funcs.insert(name.to_owned(), sym),
            }
        } else {
            None
        }
    }

    /// Lookup the given symbol name according to the scope rules.
    ///
    /// Returns `true` if the symbol is found.
    #[inline]
    pub fn check_exists(&self, name: &str) -> bool {
        self.lookup_symbol(name).is_some()
    }
}

// Visitor
#[deprecated]
impl Mapper {
    pub fn build_symbols(&mut self, tree: &CompilationUnit) -> Result<SymbolTable, Infallible> {
        self.map_block(&tree.block);
        Ok(self.take_symbols())
    }

    fn map_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            match stmt {
                Stmt::Comment(_) => { /* Ignore */ }
                Stmt::Const(def) => {
                    self.map_const_def(def);
                }
                Stmt::Var(def) => {
                    self.map_var_def(def);
                }
                Stmt::Expr(expr) => {
                    self.map_expr(expr);
                }
                Stmt::Func(func) => {
                    self.map_func_def(func);
                }
            }
        }
    }

    fn map_const_def(&mut self, const_def: &ConstDef) {
        if self.check_exists(const_def.name.as_str()) {
            panic!("symbol already defined: {}", const_def.name);
        }

        // Before adding the symbol to the table, check the right-hand-side.
        self.map_expr(&const_def.rhs);

        self.current.as_mut().unwrap().consts.insert(
            const_def.name.clone(),
            Symbol {
                name: const_def.name.clone(),
                kind: SymbolKind::Const(ConstValue::U8(0)),
                ty: ValueType::try_from(&const_def.ty.ty).unwrap_or_else(|_| panic!("unknown type {}", const_def.ty.ty)),
            },
        );
    }

    fn map_var_def(&mut self, var_def: &VarDef) {
        if self.check_exists(var_def.name.as_str()) {
            panic!("symbol already defined: {}", var_def.name);
        }

        // Before adding the symbol to the table, check the right-hand-side.
        if let Some(ref expr) = var_def.rhs {
            self.map_expr(expr);
        }

        let ty = &var_def.ty.as_ref().expect("type inferrence not implemented yet").ty;
        self.current.as_mut().unwrap().vars.insert(
            var_def.name.clone(),
            Symbol {
                name: var_def.name.clone(),
                kind: SymbolKind::Var,
                ty: ValueType::try_from(ty).unwrap_or_else(|_| panic!("unknown type '{}'", ty)),
            },
        );
    }

    fn map_func_def(&mut self, func: &FuncDef) {
        let FuncDef {
            sig: FuncSig { ident, args, .. },
            ..
        } = func;

        let arity = args
            .items
            .iter()
            .map(|delim| &delim.item.ty)
            .map(|ty_ident| ValueType::try_from(ty_ident).unwrap_or_else(|_| panic!("unknown type '{}'", ty_ident)))
            .collect::<Vec<_>>();

        let symbol = Symbol {
            name: ident.name.to_string(),
            kind: SymbolKind::Function(arity),
            // TODO: Return type
            ty: ValueType::Void,
        };

        // Build a unique key for the function.
        let mut key = String::new();
        symbol.to_func_sig(&mut key).unwrap();

        // Check whether a function with this name, arguments
        // and return type has already been declared.
        if self.lookup_symbol(key.as_str()).is_some() {
            panic!("function already defined: '{}'", key);
        }

        self.current.as_mut().unwrap().funcs.insert(key, symbol);
    }

    fn map_expr(&mut self, expr: &Expr) -> ValueType {
        match expr {
            Expr::Access(access) => self.map_expr_access(access),
            Expr::Binary(bin) => {
                let lhs_ty = self.map_expr(bin.lhs.as_ref());
                let rhs_ty = self.map_expr(bin.rhs.as_ref());

                // Type check.
                if lhs_ty != rhs_ty {
                    panic!(
                        "type error: operator '{}' cannot be applied to types '{}' and '{}'",
                        bin.operator.kind, lhs_ty, rhs_ty
                    );
                }

                lhs_ty
            }
            Expr::Unary(un) => self.map_expr(un.rhs.as_ref()),
            Expr::Literal(literal) => self.map_expr_literal(literal),
            Expr::NoOp => {
                /* Ignore */
                ValueType::Void
            }
        }
    }

    /// When an expression accesses a symbol, we ensure
    /// that it has been defined.
    fn map_expr_access(&mut self, access: &Access) -> ValueType {
        match self.lookup_symbol(access.ident.name.as_str()) {
            Some(symbol) => symbol.ty.clone(),
            None => panic!("symbol '{}' does not exist", access.ident.name.as_str()),
        }

        // TODO: Associate access with constant so bytecode emitter can inline the right value.
    }

    fn map_expr_literal(&mut self, literal: &Literal) -> ValueType {
        match literal.value {
            LitValue::U8(_) => ValueType::U8,
            LitValue::Bool(_) => ValueType::Bool,
        }
    }
}

impl Default for Mapper {
    #[inline]
    fn default() -> Self {
        Mapper::new()
    }
}