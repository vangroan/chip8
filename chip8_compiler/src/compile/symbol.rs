use crate::parsing::Ident;
use std::{collections::HashMap, convert::TryFrom};
use super::Register;

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    /// Static type known at compile time.
    pub ty: ValueType,
}

/// Static value type of the symbol known at compile time.
///
/// Compiler is pretty simple so no fancy type system.
#[derive(Debug)]
pub enum ValueType {
    /// One and only number type.
    U8,
    /// Zero and one can cover bools.
    Bool,
    /// Multiple values packed together.
    Record,
    /// 12-bit memory address.
    /// FIXME: Is this even possible? We can't fo arithmetic with 12-bit values.
    Pointer,
}

impl TryFrom<&Ident> for ValueType {
    type Error = ();

    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        match ident.name.as_str() {
            "u8" => Ok(ValueType::U8),
            "bool" => Ok(ValueType::Bool),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum SymbolKind {
    /// Constants have a value fixed at compile time.
    Const,
    /// Variables can change value during runtime.
    /// Assigned a register of its own, unique in its
    /// containing function scope.
    Var,
    Function,
}

#[derive(Debug)]
pub enum SymbolScope {
    Global,
    Local,
    Parameter,
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub var_count: usize,
    /// Constants in the global scope.
    pub consts: HashMap<String, Symbol>,
    /// Variables in the global scope.
    pub vars: HashMap<String, Symbol>,
    /// Functions in a compilation unit.
    pub funcs: HashMap<String, Symbol>,
    pub scopes: Vec<SymbolTable>,
}
